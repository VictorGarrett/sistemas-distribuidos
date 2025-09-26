import Pyro5.api
import Pyro5
import threading
import time
import sys
import queue

class Permissions:
    def __init__(self):
        self.peers = {}
        self.proxies = {}
        self.permissions = {}
        self.alive = {} 

    def add_peer(self, peer_id, host, port):
        self.peers[peer_id] = (host, port)
        self.permissions[peer_id] = False
        self.alive[peer_id] = time.time()
        self.proxies[peer_id] = Pyro5.api.Proxy(f"PYRO:peer.{peer_id}@{host}:{port}")

    def remove_peer(self, peer_id):
        if peer_id in self.peers:
            del self.peers[peer_id]
            del self.permissions[peer_id]
            del self.alive[peer_id]
            del self.proxies[peer_id]
    
    def give_permission(self, peer_id):
        self.permissions[peer_id] = True

    def check_alive(self, peer_id):
        return (time.time() - self.alive[peer_id]) < 10

    def send_heartbeats(self, id):
        for peer_id, proxy in self.proxies.items():
            try:
                proxy.receive_heartbeat(id)
            except Pyro5.errors.CommunicationError:
                print(f"Failed to send heartbeat to peer {peer_id}")
    
    def set_alive(self, peer_id):
        self.alive[peer_id] = time.time()

    def remove_dead_peers(self):
        for peer_id in list(self.peers.keys()):
            if not self.check_alive(peer_id):
                self.remove_peer(peer_id)
    
    def reset(self):
        for peer_id in self.permissions:
            self.permissions[peer_id] = False
    
    def ask_permissions(self, request_time, id):
        self.reset()
        for peer_id, proxy in self.proxies.items():
            try:
                proxy.receive_request((id, request_time,))
            except Pyro5.errors.CommunicationError:
                print(f"Failed to send request to peer {peer_id}")

    def all_granted(self):
        return all(self.permissions.values())


class Peer:

    def __init__(self, id, host, port):
        self.host = host
        self.port = port
        self.state = 'RELEASED'
        self.ns = Pyro5.api.locate_ns()
        self.id = id
        self.request_time = time.time()
        self.request_queue = []
        self.permissions = Permissions()
        self.last_heartbeat = time.time()

        peers = {}
        peer_uris = self.ns.list(prefix="peer.")
        for name, uri in peer_uris.items():
            if name != f"peer.{self.id}":
                peers[name] = Pyro5.api.Proxy(uri)
        
        print(f"Discovered peers: {peers}")
        for p_name, p_proxy in peers.items():
            p_proxy.receive_join(self.id, self.host, self.port)
            self.permissions.add_peer(int(p_name.split(".")[1]), p_proxy._pyroUri.host, p_proxy._pyroUri.port)



    def get_host(self):
        return self.host
    def get_port(self):
        return self.port
    def get_state(self):
        return self.state

    def send_response(self, req, granted):
        peer_id = req[0]
        try:
            self.permissions.proxies[peer_id].receive_response(self.id, granted)
        except Pyro5.errors.CommunicationError:
            print(f"Failed to send response to peer {peer_id}")

    def send_request(self):
        self.request_time = time.time()
        self.state = 'WANTED'
        self.permissions.ask_permissions(self.request_time, self.id)


    # Pyro interface for remote peers -----------------------------------
    @Pyro5.api.expose
    def receive_heartbeat(self, peer_id):
        command_queue.put(("heartbeat", peer_id))

    @Pyro5.api.expose
    def receive_request(self, req):
        command_queue.put(("request", req))

    @Pyro5.api.expose
    def receive_response(self, peer_id, granted):
        command_queue.put(("response", peer_id, granted))

    @Pyro5.api.expose
    def receive_join(self, peer_id, host, port):
        command_queue.put(("join", peer_id, host, port))


    # Main peer loop -------------------------
    def run(self):
        while True:
            # Send heartbeat to all peers
            if time.time() - self.last_heartbeat >= 1:
                self.permissions.send_heartbeats(self.id)
                self.last_heartbeat = time.time()

            # Check for dead peers
            self.permissions.remove_dead_peers()

            # Process commands (from CLI or Pyro)
            if not command_queue.empty():
                cmd = command_queue.get()

                # CLI string commands ------------------------
                if isinstance(cmd, str):
                    if cmd == "request resource":
                        self.send_request()
                        print("Request sent.")
                        print(f"Current state: {self.get_state()}")
                    elif cmd == "free resource":
                        if self.state == 'HELD':
                            self.state = 'RELEASED'
                            self.permissions.reset()
                            print("Resource freed.")
                        else:
                            print("Cannot free resource; resource not held.")

                # Pyro event tuples --------------------------
                elif isinstance(cmd, tuple):
                    action = cmd[0]

                    if action == "heartbeat":
                        _, peer_id = cmd
                        self.permissions.set_alive(peer_id)

                    elif action == "request":
                        _, req = cmd
                        self.request_queue.append(req)
                        print(f"Added request {req[0]}")

                    elif action == "response":
                        _, peer_id, granted = cmd
                        if granted:
                            self.permissions.give_permission(peer_id)
                            print(f"Got permission from {peer_id}")

                    elif action == "join":
                        _, peer_id, host, port = cmd
                        self.permissions.add_peer(peer_id, host, port)
                        print(f"Added peer {peer_id} ({host}:{port})")

                # Process state
                if self.state == 'RELEASED':
                    for req in self.request_queue:
                        self.send_response(req, True)
                        self.request_queue.remove(req)
                    
                elif self.state == 'WANTED':
                    for req in self.request_queue:
                        if req[1] < self.request_time or (req[1] == self.request_time and req[0] < self.id):
                            self.send_response(req, True)
                            self.request_queue.remove(req)
                    if self.permissions.all_granted():
                        self.state = 'HELD'

                elif self.state == 'HELD':
                    print("Using thing...")
                else:
                    # oh no
                    break


def run_pyro_server(peer_instance):
    daemon = Pyro5.api.Daemon(host=peer_instance.get_host(), port=peer_instance.get_port())
    service_uri = daemon.register(peer_instance, f"peer.{peer_instance.id}")

    ns = Pyro5.api.locate_ns()
    ns.register(f"peer.{peer_instance.id}", service_uri)

    daemon.requestLoop()

def run_pyro_nameserver(address, port):
    Pyro5.nameserver.start_ns_loop(host=address, port=port)

def run_cli(peer):
    """Runs a command-line interface for the peer."""
    while True:
        command = input("> ")
        if command == "request resource":
            command_queue.put("request resource")
        elif command == "free resource":
            command_queue.put("free resource")
        elif command == "list peers":
            print(f"Known peers: {peer.permissions.peers}")
        elif command == "exit":
            print("Exiting CLI...")
            break
        else:
            print("Unknown command. Available commands: request resource, free resource, list peers")


def main(peer):
    peer.run()



if __name__ == "__main__":

    command_queue = queue.Queue()

    if len(sys.argv) < 4:
        print("Usage: python peer.py <peer_id> <peer_host> <peer_port>")
        sys.exit(1) # Exit the program with an error code

    try:
        # Get the ID from the command-line arguments and convert it to an integer
        peer_id = int(sys.argv[1])
        peer_host = sys.argv[2]
        peer_port = int(sys.argv[3])
    except ValueError:
        print("Error: The peer ID and port must be valid integers.")
        sys.exit(1)

    # Start nameserver if not already running
    ns_address = "localhost"
    ns_port = 9090

    try:
        Pyro5.api.locate_ns(host=ns_address, port=ns_port)
        print(f"Nameserver found at {ns_address}:{ns_port}.")
    except Pyro5.errors.NamingError:
        nameserver_thread = threading.Thread(
            target=run_pyro_nameserver, 
            args=(ns_address, ns_port), 
            daemon=True
        )
        nameserver_thread.start()


    peer = Peer(peer_id, peer_host, peer_port)

    pyro_thread = threading.Thread(target=run_pyro_server, args=(peer,), daemon=True)
    pyro_thread.start()

    time.sleep(2)

    cli_thread = threading.Thread(target=run_cli, args=(peer,), daemon=True)
    cli_thread.start()

    #main_thread = threading.Thread(target=main, args=(peer,), daemon=True)
    #main_thread.start()
    main(peer)
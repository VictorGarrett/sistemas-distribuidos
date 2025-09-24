import Pyro5.api
import Pyro5
import threading
import time
import sys

class Permissions:
    def __init__(self):
        self.peers = {}
        self.permissions = {}
        self.alive = {} 

    def add_peer(self, peer_id, host, port):
        self.peers[peer_id] = (host, port)

    def remove_peer(self, peer_id):
        if peer_id in self.peers:
            del self.peers[peer_id]
            del self.permissions[peer_id]
            del self.alive[peer_id]
    
    def give_permission(self, peer_id):
        self.permissions[peer_id] = True

    def check_alive(self, peer_id):
        return (time.now() - self.alive[peer_id]) < 10
    
    def set_alive(self, peer_id):
        self.alive[peer_id] = time.now()

    def remove_dead_peers(self):
        for peer_id in list(self.peers.keys()):
            if not self.check_alive(peer_id):
                self.remove_peer(peer_id)
    
    def reset(self):
        for peer_id in self.permissions:
            self.permissions[peer_id] = False
            self.alive[peer_id] = time.now()
    
    def all_granted(self):
        return all(self.permissions.values())


class Peer:

    def __init__(self, id, host, port):
        self.host = host
        self.port = port
        self.state = 'RELEASED'
        self.ns = Pyro5.api.locate_ns()
        self.id = id
        self.request_queue = []
        self.permissions = Permissions()

        peers = {}
        peer_uris = self.ns.list(prefix="peer.")
        for name, uri in peer_uris.items():
            if name != f"peer.{self.id}":
                peers[name] = Pyro5.api.Proxy(uri)
        
        print(f"Discovered peers: {list(peers.keys())}")
        for p_name, p_proxy in peers.items():
            p_proxy.receive_join(self.id, self.host, self.port)


    def get_host(self):
        return self.host
    def get_port(self):
        return self.port
    def get_state(self):
        return self.state

    def send_response(self, req, granted):
        # send confirmation to the requesting peer (TODO)
        pass

    def send_request(self):
        self.state = 'WANTED'
        # send request to all peers (TODO)

    # Pyro interface for remote peers -----------------------------------

    @Pyro5.api.expose
    def receive_heartbeat(self, peer_id):
        self.permissions.set_alive(peer_id)

    @Pyro5.api.expose
    def receive_request(self, req):
       self.request_queue.append(req)
    
    @Pyro5.api.expose
    def receive_response(self, peer_id, granted):
        self.permissions.set_alive(peer_id)
        if granted:
            self.permissions.give_permission(peer_id)
    @Pyro5.api.expose
    def receive_join(self, peer_id, host, port):
        self.permissions.add_peer(peer_id, host, port)
    

    def run(self):

        self.permissions.remove_dead_peers()

        while True:
            if self.state == 'RELEASED':
                for req in self.request_queue:
                    self.send_confirmation(req)
                    self.request_queue.remove(req)
                
            elif self.state == 'WANTED':
                if not self.permissions.all_granted():
                    self.state = 'HELD'
                else:
                    time.sleep(1)
            elif self.state == 'HELD':
                print("Using thing...")
            else:
                # oh no
                break


def run_pyro_server(peer_instance):
    daemon = Pyro5.api.Daemon(host=peer_instance.get_host(), port=peer_instance.get_port())
    service_uri = daemon.register(peer_instance, f"peer.{peer_instance.id}")

    daemon.requestLoop()

def run_pyro_nameserver(address, port):
    Pyro5.nameserver.start_ns_loop(host=address, port=port)

def run_cli(peer):
    """Runs a command-line interface for the peer."""
    while True:
        command = input("> ")
        if command == "request resource":
            peer.send_request()
            print("Request sent.")
            print(f"Current state: {peer.get_state()}")
        elif command == "free resource":
            if peer.get_state() == 'HELD':
                peer.state = 'RELEASED'
                peer.permissions.reset()
                print("Resource freed.")
            else:
                print("Cannot free resource; resource not held.")
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

    main(peer)
import Pyro5.api
import threading
import time

class Permissions:
    def __init__(self):
        self.peers = {}
        self.permissions = {}

    def add_peer(self, peer_id, host, port):
        self.peers[peer_id] = (host, port)

    def remove_peer(self, peer_id):
        if peer_id in self.peers:
            del self.peers[peer_id]
            del self.permissions[peer_id]
    
    def give_permission(self, peer_id):
        self.permissions[peer_id] = True
    
    def reset(self):
        for peer_id in self.permissions:
            self.permissions[peer_id] = False
    
    def all_granted(self):
        return all(self.permissions.values())


class Peer:

    def __init__(self, host, port):
        self.host = host
        self.port = port
        self.reader = None
        self.writer = None
        self.state = 'RELEASED'
        self.id = 0
        self.request_queue = []
        self.permissions = Permissions()

    def get_host(self):
        return self.host
    def get_port(self):
        return self.port
    def get_state(self):
        return self.state

    def send_confirmation(self, req):
        # send confirmation to the requesting peer (TODO)
        pass

    def send_request(self):
        self.state = 'WANTED'
        # send request to all peers (TODO)

    # Pyro interface for remote peers -----------------------------------

    @Pyro5.api.expose
    def receive_request(self, req):
       self.request_queue.append(req)
    
    @Pyro5.api.expose
    def receive_confirmation(self, peer_id):
        self.permissions.give_permission(peer_id)
    
    

    async def run(self):

        while True:
            if self.state == 'RELEASED':
                for req in self.request_queue:
                    self.send_confirmation(req)
                    self.request_queue.remove(req)
                
            elif self.state == 'WANTED':
                while not self.permissions.all_granted():
                    time.sleep(1)
                self.state = 'HELD'
            elif self.state == 'HELD':
                print("Using thing...")
                time.sleep(5)
                print("Enough using thing.")
                self.state = 'RELEASED'
            else:
                # oh no
                break


def run_pyro_server(peer_instance):
    daemon = Pyro5.Daemon(host=peer_instance.get_host(), port=peer_instance.get_port())
    service_uri = daemon.register(peer_instance, "TODO")

    daemon.requestLoop()

def main(peer):
    peer.run()



if __name__ == "__main__":

    peer = Peer("localhost", 8888)

    pyro_thread = threading.Thread(target=run_pyro_server, args=(peer), daemon=True)
    pyro_thread.start()

    main()
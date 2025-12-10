#!/usr/bin/env python3
"""
gRPC CLI Frontend for Gateway Services
"""

import grpc
import sys
import time
import threading
from datetime import datetime

# Import generated protobuf modules (you'll generate these)
import gateway_pb2
import gateway_pb2_grpc


class GatewayClient:
    def __init__(self, host='localhost', port='50051'):
        self.channel = grpc.insecure_channel(f'{host}:{port}')
        self.auction_stub = gateway_pb2_grpc.AuctionServiceStub(self.channel)
        self.bid_stub = gateway_pb2_grpc.BidServiceStub(self.channel)
        self.event_stub = gateway_pb2_grpc.EventStreamStub(self.channel)
        self.stream_thread = None
        self.stop_stream = False

    def get_active_auctions(self):
        """Get all active auctions"""
        try:
            request = gateway_pb2.GetActiveAuctionsRequest()
            response = self.auction_stub.GetActiveAuctions(request)
            
            if not response.auctions:
                print("\nNo active auctions found.")
                return
            
            print("\n" + "="*70)
            print("ACTIVE AUCTIONS")
            print("="*70)
            for auction in response.auctions:
                status = "Active" if auction.status else "Inactive"
                start = datetime.fromtimestamp(auction.start_timestamp//1000).strftime('%Y-%m-%d %H:%M:%S')
                end = datetime.fromtimestamp(auction.end_timestamp//1000).strftime('%Y-%m-%d %H:%M:%S')
                
                print(f"\nAuction ID: {auction.id}")
                print(f"Item: {auction.item}")
                print(f"Status: {status}")
                print(f"Start: {start}")
                print(f"End: {end}")
                print("-"*70)
        except grpc.RpcError as e:
            print(f"\nError: {e.code()}: {e.details()}")

    def create_auction(self, item_name, start_timestamp, end_timestamp):
        """Create a new auction"""
        try:
            request = gateway_pb2.CreateAuctionRequest(
                item_name=item_name,
                start_timestamp=start_timestamp,
                end_timestamp=end_timestamp
            )
            response = self.auction_stub.CreateAuction(request)
            
            if response.success:
                print(f"\n✓ Auction created successfully!")
            else:
                print("\n✗ Failed to create auction")
        except grpc.RpcError as e:
            print(f"\nError: {e.code()}: {e.details()}")

    def create_bid(self, auction_id, client_id, value, signature, public_key, valid):
        """Create a new bid"""
        try:
            request = gateway_pb2.CreateBidRequest(
                auction_id=auction_id,
                client_id=client_id,
                value=value,
                signature=signature,
                public_key=public_key,
                valid=valid
            )
            response = self.bid_stub.CreateBid(request)
            
            if response.success:
                print(f"\n✓ Bid placed successfully!")
                print(f"  Auction ID: {auction_id}")
                print(f"  Client ID: {client_id}")
                print(f"  Value: ${value:.2f}")
            else:
                print("\n✗ Failed to place bid")
        except grpc.RpcError as e:
            print(f"\nError: {e.code()}: {e.details()}")

    def subscribe_to_auctions(self, client_id, auction_ids):
        """Subscribe to specific auctions"""
        try:
            request = gateway_pb2.AuctionSubscribeRequest(
                client_id=client_id,
                auction_id=auction_ids
            )
            response = self.event_stub.AuctionSubscribe(request)
            
            if response.success:
                print(f"\n✓ Subscribed to auctions: {auction_ids}")
            else:
                print("\n✗ Failed to subscribe")
        except grpc.RpcError as e:
            print(f"\nError: {e.code()}: {e.details()}")

    def unsubscribe_from_auctions(self, client_id, auction_ids):
        """Unsubscribe from specific auctions"""
        try:
            request = gateway_pb2.AuctionUnsubscribeRequest(
                client_id=client_id,
                auction_id=auction_ids
            )
            response = self.event_stub.AuctionUnsubscribe(request)
            
            if response.success:
                print(f"\n✓ Unsubscribed from auctions: {auction_ids}")
            else:
                print("\n✗ Failed to unsubscribe")
        except grpc.RpcError as e:
            print(f"\nError: {e.code()}: {e.details()}")

    def start_event_stream(self, client_id):
        """Start listening to event stream in a separate thread"""
        def stream_events():
            try:
                request = gateway_pb2.EventRequest(client_id=client_id)
                print(f"\n📡 Starting event stream for client {client_id}...")
                print("Listening for events (press 's' to stop)...\n")
                
                for event in self.event_stub.StreamStart(request):
                    if self.stop_stream:
                        break
                    
                    timestamp = datetime.now().strftime('%H:%M:%S')
                    print(f"[{timestamp}] Event: {event.event_type}")
                    print(f"  Auction ID: {event.auction_id}")
                    print(f"  Data: {event.data}")
                    print("-"*50)
                    
            except grpc.RpcError as e:
                if not self.stop_stream:
                    print(f"\nStream error: {e.code()}: {e.details()}")
        
        self.stop_stream = False
        self.stream_thread = threading.Thread(target=stream_events, daemon=True)
        self.stream_thread.start()

    def stop_event_stream(self):
        """Stop the event stream"""
        if self.stream_thread and self.stream_thread.is_alive():
            self.stop_stream = True
            print("\n📡 Stopping event stream...")
            time.sleep(0.5)
        else:
            print("\nNo active stream to stop")

    def close(self):
        """Close the gRPC channel"""
        self.stop_event_stream()
        self.channel.close()


def print_menu():
    """Print the main menu"""
    print("\n" + "="*70)
    print("GATEWAY GRPC CLIENT")
    print("="*70)
    print("\nAuction Commands:")
    print("  1. Get Active Auctions")
    print("  2. Create Auction")
    print("\nBid Commands:")
    print("  3. Place Bid")
    print("\nEvent Stream Commands:")
    print("  4. Subscribe to Auctions")
    print("  5. Unsubscribe from Auctions")
    print("  6. Start Event Stream")
    print("  s. Stop Event Stream")
    print("\nOther:")
    print("  q. Quit")
    print("="*70)


def get_input(prompt, input_type=str):
    """Get user input with type conversion"""
    while True:
        try:
            value = input(prompt)
            if value.lower() == 'q':
                return None
            return input_type(value)
        except ValueError:
            print(f"Invalid input. Please enter a valid {input_type.__name__}")


def main():
    # Get connection details
    host = input("Gateway host (default: localhost): ").strip() or "localhost"
    port = input("Gateway port (default: 50051): ").strip() or "50051"
    
    client = GatewayClient(host, port)
    client_id = None
    
    try:
        while True:
            print_menu()
            choice = input("\nEnter command: ").strip().lower()
            
            if choice == 'q':
                print("\nGoodbye!")
                break
            
            elif choice == '1':
                # Get Active Auctions
                client.get_active_auctions()
            
            elif choice == '2':
                # Create Auction
                print("\n--- Create Auction ---")
                item_name = get_input("Item name: ", str)
                if item_name is None:
                    continue
                
                start_ts = get_input("Start timestamp (Unix epoch): ", int)
                if start_ts is None:
                    continue
                
                end_ts = get_input("End timestamp (Unix epoch): ", int)
                if end_ts is None:
                    continue
                
                client.create_auction(item_name, start_ts, end_ts)
            
            elif choice == '3':
                # Place Bid
                print("\n--- Place Bid ---")
                auction_id = get_input("Auction ID: ", int)
                if auction_id is None:
                    continue
                
                if client_id is None:
                    client_id = get_input("Your Client ID: ", int)
                    if client_id is None:
                        continue
                
                value = get_input("Bid value: ", float)
                if value is None:
                    continue
                
                signature = get_input("Signature: ", str)
                if signature is None:
                    continue
                
                public_key = get_input("Public key: ", str)
                if public_key is None:
                    continue
                
                valid = get_input("Valid (True/False): ", lambda x: x.lower() == 'true')
                if valid is None:
                    continue
                
                print(f"auction {auction_id}, client {client_id}, value {value}")
                client.create_bid(auction_id, client_id, value, signature, public_key, valid)
            
            elif choice == '4':
                # Subscribe to Auctions
                print("\n--- Subscribe to Auctions ---")
                if client_id is None:
                    client_id = get_input("Your Client ID: ", int)
                    if client_id is None:
                        continue
                
                auction_ids_str = get_input("Auction IDs (comma-separated): ", str)
                if auction_ids_str is None:
                    continue
                
                auction_ids = [int(x.strip()) for x in auction_ids_str.split(',')]
                client.subscribe_to_auctions(client_id, auction_ids)
            
            elif choice == '5':
                # Unsubscribe from Auctions
                print("\n--- Unsubscribe from Auctions ---")
                if client_id is None:
                    client_id = get_input("Your Client ID: ", int)
                    if client_id is None:
                        continue
                
                auction_ids_str = get_input("Auction IDs (comma-separated): ", str)
                if auction_ids_str is None:
                    continue
                
                auction_ids = [int(x.strip()) for x in auction_ids_str.split(',')]
                client.unsubscribe_from_auctions(client_id, auction_ids)
            
            elif choice == '6':
                # Start Event Stream
                if client_id is None:
                    client_id = get_input("Your Client ID: ", int)
                    if client_id is None:
                        continue
                
                client.start_event_stream(client_id)
            
            elif choice == 's':
                # Stop Event Stream
                client.stop_event_stream()
            
            else:
                print("\nInvalid choice. Please try again.")
            
            input("\nPress Enter to continue...")
    
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    finally:
        client.close()


if __name__ == "__main__":
    main()
build:
	cargo build --release

run-server:
	# Server mode: We bind to 0.0.0.0 and DO NOT specify a peer
	sudo ./target/release/ghost_tunnel --bind 0.0.0.0:8000 --tun-ip 10.0.0.1

run-client:
	# Client mode: We bind to 8001 and point to the server at port 8000
	sudo ./target/release/ghost_tunnel --bind 127.0.0.1:8001 --peer 127.0.0.1:8000 --tun-ip 10.0.0.2

test-chaos:
	sudo ./scripts/simulate_loss.sh
# Distributed Systems Project (rustinpeace)

# Project Overview
This project implements the base infrastructure for a distributed computing system.

# Team
César Noé Ascencio Palma      
Andrea Delgadillo Becerra      
Álvaro Guzmán Cabrera         
Nora Esthela Simental Beaven  
- Each member works on an independent Linux virtual machine (CLI only)

# Technologies
- Linux
- WireGuard
- Docker & Docker Compose
- Rust

# Network Architecture
- VPN topology: hub-and-spoke
- VPN netork: 10.10.10.0/24
- One central hub node and three peer nodes
- All comunication occurs exclusively over the VPN

## Container Infrastructure
- Docker image defined using a reproducible Dockerfile
- Generic worker containers (no distributed logic yet)
- Minimum of four worker containers per node
- No ports exposed to the public network
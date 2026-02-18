# Proyecto de sistemas distribuido (rustinpeace)

# Descripción General
Este proyecto implementa la infraestructura base para un sistema de cómputo distribuido desplegado sobre una red privada virtual (VPN).

La arquitectura simula un entorno distribuido real donde múltiples nodos independientes se comunican exclusivamente a través de una red segura basada en WireGuard. Cada nodo ejecuta cargas de trabajo en contenedores Docker, garantizando aislamiento, reproducibilidad y escalabilidad.

El objetivo principal es construir una base sólida para futuros mecanismos de coordinación distribuida, procesamiento paralelo y comunicación segura entre nodos.

# Equipo
César Noé Ascencio Palma      
Andrea Delgadillo Becerra      
Álvaro Guzmán Cabrera         
Nora Esthela Simental Beaven

- Cada integrante trabaja sobre una máquina virtual Linux independiente (modo CLI únicamente) simulando nodos distribuidos reales en un entorno de producción

# Tecnologías
- Linux
- WireGuard
- Docker & Docker Compose
- Rust

# Arquitectura de la red
Topología tipo hub-and-spoke (estrella):
    -Un nodo central (hub)
    -Tres nodos peer (trabajadores)
    -Todos conectados mediante túneles WireGuard

# Infraestructura de contenedores
Cada nodo ejecuta múltiples contenedores con las siguientes características:
    Imagen Docker reproducible definida mediante Dockerfile
    Contenedores tipo “worker” genéricos (sin lógica distribuida implementada aún)
    Mínimo de cuatro contenedores worker por nodo
    Sin puertos expuestos a la red pública
    Comunicación interna únicamente dentro de la VPN

Esta arquitectura permite:
    Escalabilidad horizontal
    Replicabilidad del entorno
    Separación clara entre infraestructura y lógica distribuida
    Simulación realista de un sistema distribuido seguro

## 
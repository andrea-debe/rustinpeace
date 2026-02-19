# Proyecto de sistemas distribuido (rustinpeace)

## Descripción General
Este proyecto implementa la infraestructura base para un sistema de cómputo distribuido desplegado sobre una red privada virtual (VPN).

La arquitectura simula un entorno distribuido real donde múltiples nodos independientes se comunican exclusivamente a través de una red segura basada en WireGuard. Cada nodo ejecuta cargas de trabajo en contenedores Docker, garantizando aislamiento, reproducibilidad y escalabilidad.

El objetivo principal es construir una base sólida para futuros mecanismos de coordinación distribuida, procesamiento paralelo y comunicación segura entre nodos.

## Equipo
César Noé Ascencio Palma      
Andrea Delgadillo Becerra      
Álvaro Guzmán Cabrera         
Nora Esthela Simental Beaven

- Cada integrante trabaja sobre una máquina virtual Linux independiente (modo CLI únicamente) simulando nodos distribuidos reales en un entorno de producción

## Tecnologías
- Linux
- WireGuard
- Docker & Docker Compose
- Rust

## Instrucciones para levantar la VPN
# 1. Instalar WireGuard en cada nodo:
sudo apt update
sudo apt install wireguard -y

# 2. Generar claves en cada nodo:
wg genkey | tee privatekey | wg pubkey > publickey

# 3.Configurar el archivo /etc/wireguard/wg0.conf:
- Asignar una IP del segmento 10.00.00.0/24
- Definir la clave privada del nodo
- En el nodo hub: agregar los peers con AllowedIPs = 10.00.00.X/32
- En los peers: apuntar al hub con Endpoint = IP_PUBLICA:51820
- Usar AllowedIPs = 10.00.00.0/24 en los peers

# 4. Levantar la VPN:
sudo wg-quick up wg0

# 5. Verificar la conexión:
sudo wg
ping 10.00.00.X

## Especificaciones del Sistema
## Requisitos de Hardware
Para la correcta implementación del sistema distribuido se requiere:
- Mínimo 4 máquinas virtuales o equipos físicos (1 nodo hub y 3 nodos peer)
- 2 GB de RAM por nodo (recomendado 4 GB)
- 2 núcleos de CPU por nodo
- 20 GB de almacenamiento disponible por nodo
- Conectividad a Internet para establecimiento inicial de la VPN

# Requisitos de Software

- Cada nodo debe contar con:
- Sistema operativo Linux (distribución compatible con systemd)
- WireGuard instalado y configurado
- Docker Engine instalado
- Docker Compose instalado
- Acceso a terminal (CLI)
- Permisos de superusuario (sudo)

## Configuración de Red

- Red privada virtual basada en WireGuard
- Segmento de red VPN: 10.10.10.0/24
- Topología: hub-and-spoke
- Cifrado de extremo a extremo mediante claves públicas/privadas
- Sin exposición de puertos al exterior
- Todo el tráfico inter-nodo restringido a la VPN

## Configuración de Contenedores

- Imagen Docker reproducible definida mediante Dockerfile
- Mínimo de cuatro contenedores worker por nodo
- Sin puertos publicados hacia la red pública
- Comunicación entre contenedores limitada al entorno interno del nodo y a la red VPN
- Orquestación mediante Docker Compose

## Consideraciones de Seguridad

- Todo el tráfico entre nodos viaja cifrado a través de WireGuard
- No se permite comunicación directa fuera de la VPN
- No se exponen servicios al internet público
- Segmentación de red estricta
- Aislamiento de procesos mediante contenedores

## Arquitectura de la red
Topología tipo hub-and-spoke (estrella):
- Un nodo central (hub)
- Tres nodos peer (trabajadores)
- Todos conectados mediante túneles WireGuard

## Infraestructura de contenedores
Cada nodo ejecuta múltiples contenedores con las siguientes características:
- Imagen Docker reproducible definida mediante Dockerfile
- Contenedores tipo “worker” genéricos (sin lógica distribuida implementada aún)
- Mínimo de cuatro contenedores worker por nodo
- Sin puertos expuestos a la red pública
- Comunicación interna únicamente dentro de la VPN

## 
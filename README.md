# Mandelbrot Distribuido: Cómputo Paralelo con Rust, gRPC y WireGuard

## Equipo
César Noé Ascencio Palma      
Andrea Delgadillo Becerra      
Álvaro Guzmán Cabrera         
Nora Esthela Simental Beaven

## Descripción general del proyecto
Este proyecto implementa un sistema de cómputo distribuido y tolerante a fallos para la generación del fractal de Mandelbrot en alta resolución. Utiliza una arquitectura **Coordinador-Trabajador** donde un nodo central distribuye fragmentos de la imagen (filas) a múltiples nodos de cómputo a través de llamadas a procedimientos remotos (**gRPC**). 

Toda la comunicación entre los nodos está asegurada y enrutada a través de una red privada virtual (**WireGuard**), simulando un entorno de nube privada. 

El sistema incorpora mecanismos de resiliencia como *Visibility Timeout* y *Heartbeats* para reasignar tareas automáticamente si un nodo trabajador (contenedor Docker) falla o se desconecta abruptamente.

## Requisitos de software
Para desplegar y ejecutar este proyecto en los distintos nodos, se requiere tener instalado:
* **Rust y Cargo:** (Edición 2021 o superior) para la compilación del código fuente.
* **Docker y Docker Compose:** Para la contenerización y orquestación de los workers.
* **WireGuard:** Para establecer el túnel VPN.
* **Git:** Para el control de versiones y sincronización de código.
* **SO Compatible:** Ubuntu Server / Linux (nativo o vía WSL2) para los nodos de cómputo, y Windows/Linux para el servidor VPN.

## Instrucciones para levantar la VPN
El clúster opera bajo la subred privada `10.0.0.0/24`. La topología de red separa la capa de enrutamiento de la capa de aplicación.

1. **Servidor VPN (HUB Red - 10.0.0.1):** * Configurar la interfaz en la máquina host (Windows/Linux) para escuchar en el puerto UDP `51820`.
   * **Importante:** Es obligatorio tener habilitado el *IP Forwarding* en este equipo para permitir que los paquetes transiten entre los Peers y el Coordinador.
2. **Nodos de Cómputo (Coordinador y Workers - 10.0.0.X):**
   * Instalar WireGuard e importar el archivo `wg0.conf` respectivo.
   * Asegurarse de que el parámetro `AllowedIPs` incluya `10.0.0.0/24` para enrutar correctamente el tráfico interno del clúster.
   * Llenar los campos de Llaves de Wireguard con los propios de la infraesctructura
   * Levantar la interfaz ejecutando: `sudo wg-quick up wg0`.
   * Verificar conectividad haciendo ping al servidor VPN (`ping 10.0.0.1`).

## Instrucciones para compilar y ejecutar el sistema distribuido en Rust
El nodo **HUB Coordinator** (IP `10.0.0.2`) es el cerebro del sistema. No ejecuta contenedores, solo administra el estado de la red y el ensamblaje de la imagen.

1. Clonar el repositorio y acceder a la carpeta del proyecto en Rust:
   ```bash
   git clone <URL_DEL_REPOSITORIO>
   cd rustinpeace/rust
   
   ```
2. Compilar y ejecutar el Coordinador optimizado para producción:
   ```bash
   cargo run --bin coordinator
   
   ```
3. El Coordinador comenzará a escuchar peticiones gRPC en `0.0.0.0:3000` y mostrará un mensaje de confirmación.

## Instrucciones para desplegar contenedores (Workers)
Los nodos **PEERS** (ej. `10.0.0.3`, `10.0.0.4`, `10.0.0.5`) aportan el poder de cómputo.

1. Acceder a la carpeta de los contenedores en Docker:
   ```bash
   cd rustinpeace/docker
   
   ```
2. Levantar el enjambre de contenedores (4 por defecto):
   ```bash
   docker compose up -d --build
   
   ```

**Nota:** Para detener temporalmente los workers sin destruir el progreso: `docker compose stop``

## Notas importantes y supuestos
* **Telemetría y Observabilidad:** El sistema utiliza variables de entorno del sistema operativo (HOSTNAME) o reloj atómico del sistema para asignar un ID único a cada contenedor, lo que permite al Coordinador contabilizar el número exacto de workers activos en tiempo real.
* **Tolerancia a Fallos:** El Coordinador maneja un Visibility Timeout estricto de 15 segundos. Si un contenedor muere (SIGKILL) o la red sufre una caída severa que exceda este tiempo, la tarea en progreso se considera huérfana y es reasignada automáticamente a un nodo sano para evitar deadlocks y pérdida de datos.
* **Variables quemadas en código:** Se asume que la IP del Coordinador (10.0.0.2) y el puerto gRPC (3000) son estáticos y conocidos por todos los workers en el momento de la compilación. Lo mismo ocurre para la resolución de la imagen, color y número de iteraciones maximas. Esto puede ser editado directamente dentro `rustinpeace/rust/src/bin/coordinator.rs`


## 

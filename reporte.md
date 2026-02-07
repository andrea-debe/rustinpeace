### Preparación de entorno de contenedores

## Responsable: Palma

# Preparación del entorno base
Cada integrante configuró una máquina linux en modo CLI, cumpliendo así con la restricción de operar sin GUI. Se utilizó la distribución Ubuntu ejecutándose en un entorno Linux compatible con conectividad a internet y las herramientas básicas de red disponibles

Previo al despliegue de contenedores, se verificó el correcto funcionamiento del sistema y se procedió a la instalación y validación de Docker y Docker Compose, herramientas que serán utilizadas para ejecutar los workers del sistema distribuido

La instalación fue validada mediante los comandos docker --version y docker compose version, confirmando la disponibilidad del motor de contenedores y del orquestador local. Adicionalmente, se ejecutó un contenedor de prueba para asegurar la correcta comunicación con el daemon de Docker

# Definición de la imagen base del worker

Con el objetivo de preparar una infraestructura reproducible y ligera, se definió una imagen base de contenedor utilizando un archivo Dockerfile. Esta imagen se construye a partir de Alpine Linux, siguiendo el principio de mínima superficie, e incluye únicamente utilidades básicas necesarias para pruebas iniciales y depuración

El contenedor worker definido en esta etapa no implementa aún lógica distribuida; su propósito es validar el entorno de ejecución, la estabilidad del despliegue y la capacidad de escalar múltiples instancias por nodo. El comando de ejecución del contenedor mantiene un proceso activo que permite verificar su correcta inicialización y operación continua

# docker-compose.yml

Para la gestión de múltiples contenedores, se creó un archivo docker-compose.yml base, el cual describe un servicio genérico denominado worker. Este archivo permite construir la imagen localmente y levantar múltiples instancias del contenedor de manera sencilla y reproducible

# Documentación inicial del repositorio
Se creó un archivo README.md inicial dentro del repositorio del proyecto. Este documento describe de forma general el objetivo del sistema, las tecnologías utilizadas, la arquitectura de red planteada y el estado actual del desarrollo

El repositorio sigue la estructura mínima solicitada, separando claramente la configuración de la VPN, los archivos relacionados con Docker, el código fuente en Rust y la documentación del proyecto. No se incluyen llaves privadas, credenciales ni evidencias gráficas, asegurando buenas prácticas de seguridad y versionamiento

# Estado
- Docker y Docker Compose instalados y verificados
- Imagen base de worker definida mediante Dockerfile
- Archivo docker-compose.yml base funcional
- Cuatro contenedores worker ejecutándose correctamente
- Documentación inicial del proyecto disponible en el repositorio

### Queda pendiente hacer meter las cosas en el reporte final, esperaré a la siguiente reunión para ponernos de acuerdo ###
### Fin del documento A y + o - el D, cualquier cosa me avisan jaja ###


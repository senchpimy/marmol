# Marmol

![Badge](https://img.shields.io/badge/License-MIT-blue)
![Badge](https://img.shields.io/badge/Version-0.1.0-green)
![Badge](https://img.shields.io/badge/Rust-1.70+-orange)

Marmol es una aplicación de escritorio multiplataforma desarrollada en Rust con `eframe` y `egui` para la gestión y edición de diferentes tipos de archivos dentro de "vaults" (carpetas protegidas o proyectos).

![imagen]()

---

## Características principales

- Soporte para múltiples tipos de archivo: Markdown, Income (ingresos), y Tasks (tareas).
- Gestión de "vaults" con múltiples archivos y configuración personalizada.
- Interfaz moderna con paneles y pestañas para navegación.
- Sistema de guardado automático del estado y configuración al cerrar la app.
- Vista y edición de archivos, incluyendo creación de nuevos archivos con plantillas iniciales.
- Posibilidad de cambiar configuraciones como tamaño de fuente, orden de archivos y más.
- Módulos separados para funcionalidades específicas (configuraciones, gráficos, búsqueda, servidor, etc.).

---

## Uso

Compila y ejecuta con:

```bash
cargo run --release
````

Esto lanzará la aplicación con interfaz gráfica nativa basada en `egui`.

---

## TODO

* Backup server

* Canvas

* Color scheme/themes

* Tags en gráficos

* Color de tags

* Movimiento de puntos en gráficos

* Mover cursor al formatear

* Enlaces entre notas

* Navegar entre directorios

* Renombrar/eliminar dirs/archivos

* Calendario

* Atajos de teclado

* Plantillas

* Búsqueda por metadata

* Iconos en explorador de archivos

* Búsqueda dentro de archivos

* Cambio checkbox con tamaño de fuente

* Añadir fuente personalizada

* Manejar ausencia de archivo actual

* Crear y guardar archivo la primera vez

* Copiar archivos entre vaults

---

## Licencia

Este proyecto está bajo la licencia MIT. Para más detalles, revisa el archivo [LICENSE](./LICENSE).

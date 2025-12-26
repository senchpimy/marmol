### 1. WikiLinks (`[[Enlace Interno]]`) - **La más importante**
Obsidian no es nada sin la capacidad de enlazar notas usando doble corchete. Actualmente usas Markdown estándar, pero el usuario de Obsidian espera escribir `[[` y que aparezca un autocompletado de sus archivos.

*   **El Reto:** `egui_commonmark` (que usas para visualizar) renderiza Markdown estándar (`[link](url)`).
*   **La Solución:**
    1.  **En el Visor:** Necesitas un pre-procesador que busque `[[Nombre de Archivo]]` en el texto y lo convierta a `[Nombre de Archivo](Nombre de Archivo)` antes de pasarlo al renderizador.
    2.  **En el Click:** Interceptar el click del enlace. Si el enlace no empieza con `http`, buscar ese archivo en tu `vault_vec` y abrirlo en una nueva pestaña.
    3.  **En el Editor:** Detectar cuando el usuario escribe `[[` y abrir un `Popup` con la lista de archivos para autocompletar.

### 2. Command Palette (Paleta de Comandos) `Ctrl + P`
Obsidian se puede usar casi sin mouse. Necesitas un menú modal que aparezca en el centro y permita ejecutar acciones.

*   **Cómo implementarlo:**
    *   En `main.rs`, detecta `ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P))`.
    *   Si se activa, dibuja una `egui::Window` flotante en el centro.
    *   Contenido: Un `TextEdit` con foco automático y debajo una lista filtrable de acciones: "Crear archivo", "Cambiar a modo oscuro", "Abrir Grafo", "Dividir pestaña".

### 3. ~Quick Switcher (Cambio Rápido) `Ctrl + O`~
Similar a la paleta de comandos, pero exclusivamente para buscar y abrir archivos rápidamente sin usar el árbol de directorios lateral.

*   **Lógica:**
    *   Igual que el anterior, pero la lista a filtrar es tu vector de archivos (`entrys_vec`).
    *   Al dar Enter, abre el archivo seleccionado.

### 5. Backlinks (Enlaces Entrantes)
Saber qué notas apuntan a la nota actual.

*   **Implementación:**
    *   Necesitas indexar tus archivos (puede ser al inicio o en segundo plano).
    *   Buscar en todos los archivos del vault quién contiene el string `[[Nombre de tu archivo actual]]`.
    *   Mostrar una lista al final del modo `View` o en un panel lateral derecho.

### 7. Soporte de Tags (`#etiqueta`)
*   **Visualización:** Detectar palabras que empiecen con `#` y pintarlas de otro color (como un botón píldora).
*   **Funcionalidad:** Al hacer clic en un tag, llevarte a la pestaña de Búsqueda (`LeftTab::Search`) con ese tag pre-llenado.

### 8. Vizualizar en lado izquierdo

Cuando se esta en un archivo hay una vizualizacion del archivo y sus enlaces
(local graph / command palette)

- profundida que tan profunda llega la coneccion de nodo A -> B -> C
- incoming/outgoing links
- 


### 9. Grafo
  - ~Flechas (direccion)~
  - ~Disaparicion del texto~
  - ~Tamaño del nodo~
  - ~Grosor de la linea~
  - ~Cuando se pone onhover en un nodo se fadeout opacity ~30% los demas y los que no
  estan conectados~
  - fuerzas
  - filtros exclusivos
  - abrir el grafo y tener el filtro ya activo
  - ~Tags toogle en el principal (un nodo grandote en el cual todos los del mismo tag estan conectados)~



### Bookmarks

### ~Mover dentro de folders~

### web clipper
Abre una template segun que pagina web se abrio [[https://www.youtube.com/watch?v=O7vGsBghWfc]]

### Obsidian databases

### Double shif: buscar/encontrar una nota (en el quickswitcher)

### Auto template triger

### map view ??? (ver mapas)

### canvas

### Calendario

### ~iconazie~

### file color

### colored tags

### ~file count~

### pinear folder

### ninja cursor?

### tags not done

### custom css en clases del archivo y aplicar estilo por archivo

### spaced repetition

### advanced slides

### templater

### homepage

### completr

### ~~kanban~~

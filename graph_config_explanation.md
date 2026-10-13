# Configuración del grafo de nodos

El grafo replica la vista de grafo de Obsidian: fuerzas, filtros, grupos de
color y opciones de visualización. Los ajustes se persisten por vault en
`<vault>/.marmol/graph.json`, de forma equivalente al `.obsidian/graph.json`
de Obsidian.

## Sintaxis de búsqueda / consultas de grupo

Tanto el filtro como los grupos aceptan los mismos operadores que Obsidian:

| Operador    | Descripción                                             |
|-------------|---------------------------------------------------------|
| `file:`     | Busca por nombre de archivo                              |
| `path:`     | Busca por ruta relativa                                  |
| `tag:`      | Busca por tag                                            |
| `section:`  | Busca encabezados (`# ...`)                              |
| `line:`     | Busca dentro del contenido                               |
| `content:`  | Busca dentro del contenido                               |
| `texto`     | Búsqueda libre (nombre, ruta, tag o contenido)           |

Se admiten comillas para valores con espacios, p. ej. `path:"mi carpeta"`.
Varios términos se combinan con AND.

## Fichero `graph.json`

```json
{
  "center_force": 0.15,
  "repel_force": 30.0,
  "link_force": 0.8,
  "link_distance": 30.0,
  "filter_query": "",
  "show_attachments": true,
  "show_existing_only": false,
  "show_orphans": true,
  "show_tags": false,
  "show_arrows": false,
  "text_zoom_threshold": 500.0,
  "node_size": 7.0,
  "line_thickness": 2.0,
  "custom_groups": [
    { "query": "tag:rust", "color": [222, 165, 132, 255], "enabled": true }
  ]
}
```

### Campos

* `center_force`: atracción hacia el centro.
* `repel_force`: repulsión entre nodos.
* `link_force`: rigidez de los enlaces.
* `link_distance`: distancia de reposo de los enlaces (0 = automática).
* `filter_query`: consulta de filtrado activa.
* `show_attachments`: mostrar adjuntos.
* `show_existing_only`: ocultar nodos fantasma (enlaces a archivos inexistentes).
* `show_orphans`: mostrar nodos huérfanos.
* `show_tags`: mostrar nodos de tags.
* `show_arrows`: dibujar flechas direccionales.
* `text_zoom_threshold`: umbral de zoom para desvanecer las etiquetas.
* `node_size`: tamaño base de los nodos (escala con el grado de conexión).
* `line_thickness`: grosor de los enlaces.
* `custom_groups`: grupos de color con su consulta y estado.

## Compatibilidad

Se sigue leyendo el antiguo `./test.json` (`showOrphans`, `showTags`,
`colorGroups`, `centerStrength`, `repelStrength`) para mantener compatibilidad
con configuraciones previas.

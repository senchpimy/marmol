# Graph Configuration File (`test.json`)

The `test.json` file is used to configure the appearance and behavior of the graph. It has the following fields:

*   `showOrphans`: A boolean value (`true` or `false`) that determines whether to show nodes that don't have any tags.
*   `showTags`: A boolean value (`true` or `false`) that determines whether to show the tags of the nodes.
*   `colorGroups`: An array of objects, where each object defines a color for a specific tag.
    *   `tag`: The name of the tag (e.g., `"rust"`).
    *   `rgb`: An array of three numbers (from 0 to 255) representing the RGB color for the tag (e.g., `[222, 165, 132]`).
*   `centerStrength`: A number that controls the force that pulls the nodes towards the center of the graph.
*   `repelStrength`: A number that controls the force that pushes the nodes away from each other.

## Example `test.json` file:

```json
{
  "showOrphans": true,
  "showTags": true,
  "colorGroups": [
    {
      "tag": "rust",
      "rgb": [222, 165, 132]
    },
    {
      "tag": "javascript",
      "rgb": [247, 223, 30]
    }
  ],
  "centerStrength": 10,
  "repelStrength": 15
}
```

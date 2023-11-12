# `bevy_text3d`

Prototype of 3D text using `ab_glyph` to get the glyph curves from the fonts, `lyon` to generate meshes for each glyph, and `glyph_brush_layout` to position the glyphs.
Each glyph is an entity spawned under the parent `Text3dBundle` entity.

```rs
use plugin::{Text3dBundle, Text3dPlugin};
    commands.spawn(Text3dBundle {
        text: Text::from_sections([
            TextSection::new(
                "Hello,\nWorld!",
                TextStyle {
                    font: asset_server.load("fonts/Fira_Mono-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.8, 0.9, 0.7),
                },
            ),
        ])
        .into(),
        ..default()
    });
```

https://github.com/tigregalis/bevy_text3d/assets/38416468/67838da1-1dec-453a-a250-49c7b1b4622a

## To do

- [ ] librarify this
- [ ] colours don't work properly
- [ ] Text3dSize
- [ ] Text Bounds (support text wrapping)
- [ ] perhaps custom material handles injected into Text instead of colour (would have to run our own SectionText)
- [ ] double-sided mesh
- [ ] extruded mesh

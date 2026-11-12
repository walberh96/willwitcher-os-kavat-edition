# Willwitcher-OS Kavat Edition — Decisiones pendientes

Actualizado el 12 de noviembre de 2026 · [Documento de diseño](DESIGN.md) · [Mockup interactivo](mockup/index.html)

## Cómo funciona

Aquí está solo lo que espera tu decisión. Todo lo demás vive en el [documento de diseño](DESIGN.md) y en el mockup.

- **Si apruebas algo**, pasa al documento de diseño y al mockup, y sale de aquí.
- **Si lo rechazas**, se borra de aquí y no se vuelve a preguntar.
- **Para contestar**, basta el número por chat («D32.1 sí, D32.4 no») o escribir la respuesta junto a la decisión en este archivo.

## Resumen

Hay una decisión pendiente, D32, del 12 de noviembre de 2026: que los iconos, el cursor y lo que queda fuera sigan el tema del sistema. Está dibujada en el mockup, en la página Appearance del panel; las capturas son `settings-icons.png`, `settings-icons-tinted.png` y `settings-icons-line.png`, en `docs/screenshots/`.

El repaso de Endgame, de D22 a D31, se resolvió el 12 de noviembre de 2026. Quedaron aprobadas y ya están en el documento de diseño: las utilidades de terminal, 7zip y NTFS, headsetcontrol, Rust en Development, Marksman, y OnlyOffice como casilla sin marcar. El teclado Keychron se configura con VIA web, sin instalar qmk ni via. Quedaron rechazadas: ffmpeg y unar como paquetes propios, Deno, Signal, LibreOffice, pandoc, TeX Live, pass, Papirus tal cual, Lutris, gamemode y gemini-cli.

## D32 · Iconos, cursor y lo demás siguen el tema

Hoy el sistema de temas genera colores para las apps, pero no dice nada de los iconos ni del cursor. Propuesta: los dos se generan desde los mismos tokens, así que cambian con la paleta, el modo y el acento como el resto.

| # | Qué se propone | Recomendación | Si dices que no |
| --- | --- | --- | --- |
| D32.1 | **Iconos generados (`kavat-icons`).** Un juego de iconos en SVG que `kavat-theme` vuelve a colorear en cada cambio de tema. El estilo se elige entre los tres de abajo | Sí | Un tema de iconos fijo, que no cambia con el tema |
| D32.2 | **Cursor generado (`kavat-cursor`).** El relleno sigue al modo (claro sobre oscuro y al revés) y el borde lleva el acento. Tamaños 24, 32 y 48 | Sí | Un cursor fijo, igual en todos los temas |
| D32.3 | **Iconos de archivo en la terminal.** Los iconos de Yazi, Neovim y `kav` usan los colores de la paleta en vez de un color fijo por tipo de archivo | Sí | Cada tipo de archivo conserva su color de siempre |
| D32.4 | **Bandeja en un solo color.** Los iconos de la bandeja de la barra se pintan en el color del texto | Sí | Cada app pinta su icono de bandeja con sus colores |
| D32.5 | **Lista de lo que no sigue el tema.** La página Appearance dice qué apps no se pueden tematizar, como Steam u OnlyOffice, en vez de callarlo | Sí | No se avisa; se nota al abrir la app |

**Los tres estilos de D32.1.** Elige uno como estilo de serie; los otros dos pueden quedar como opción o quitarse.

| Estilo | Cómo es | A favor | En contra |
| --- | --- | --- | --- |
| Colour | Cada app conserva su color de marca; las carpetas y los iconos de sistema llevan el acento | Las apps se reconocen de un vistazo | Es el que menos sigue al tema |
| Tinted | Todos los iconos se pintan con colores de la paleta, sobre un fondo suave | Es el que más sigue al tema | Las apps se distinguen menos, y un icono de una app que no esté en el juego desentona |
| Line | Iconos de línea en el color del texto, con el acento en las carpetas | El más sobrio; nunca choca | Todo se parece; cuesta encontrar una app por su icono |

- **Recomendación**: Tinted de serie, porque es lo que pediste, que también sigan el tema, con Colour como opción para cuando prefieras reconocer las apps por su color.

Tres cosas que conviene saber:

- **De dónde salen los iconos**: propongo partir de Papirus, que es libre y cubre miles de apps, y recolorearlo. Recolorear las carpetas y los iconos de sistema es directo; recolorear bien los iconos de app para Tinted es trabajo de verdad, y no sé cómo de bien queda hasta probarlo.
- **Apps fuera del juego**: una app cuyo icono no esté en Papirus trae el suyo, sin tematizar, en cualquiera de los tres estilos.
- **Cursor en apps abiertas**: Hyprland cambia el cursor en vivo, pero una app ya abierta puede seguir con el anterior hasta reabrirla. Se comprueba en la fase 3.

# Willwitcher-OS Kavat Edition

Distribución personal basada en Arch Linux. Está en **fase 0, diseño**: no hay nada implementado. El siguiente paso es la fase 1, el perfil de archiso y el repo `[kavat]`, que se compilan en el PC con Arch.

## Idiomas

- Con el usuario se habla en español.
- `README.md` y el mockup van en inglés. `docs/DESIGN.md` y `reports/` van en español.

## Dónde vive el diseño

Son cuatro sitios, y los tres primeros deben decir lo mismo:

1. **Documento de diseño en claude.ai.** Es la fuente: el diseño se edita ahí.
2. **`docs/DESIGN.md`.** Es una exportación de ese documento. No se edita a mano; se regenera con `tools/sync-design.py`.
3. **`docs/mockup/index.html`.** El mockup interactivo de la interfaz, en un solo HTML. Cuando el usuario pide un «diseño propuesto» quiere verlo dibujado ahí, no descrito en texto.
4. **Documento «Decisiones pendientes» en claude.ai.** Lo que espera una decisión del usuario.

El usuario llama «artifact» tanto a los documentos de claude.ai como al mockup. Los enlaces a los dos documentos de claude.ai no están en el repo: si no los tienes, pídeselos.

## Cómo se cambia el diseño

1. Lo que el usuario tiene que decidir va al documento de decisiones pendientes, no al de diseño. Cada punto lleva un número (D1 a D9 ya se usaron; se sigue en D10), una recomendación y qué pasa si dice que no.
2. Antes de trabajar, se mira ese documento: el usuario contesta ahí, cambiando el desplegable de cada decisión, o por chat.
3. Lo aprobado pasa al documento de diseño, a `docs/DESIGN.md`, al mockup y, si toca, al README, y sale de pendientes.
4. Lo rechazado se borra de pendientes y no se vuelve a preguntar. Si el usuario aprueba una parte y dice «lo otro no», ese no cubre todo lo demás que estuviera sobre la mesa.
5. Tras tocar el mockup se rehacen las capturas afectadas con `tools/screenshots.sh <nombre>`.
6. Se hace commit en `main` después de cada cambio de diseño, sin preguntar. El push no va incluido: se hace cuando el usuario lo pide.

## Criterios del usuario

- No complicar: no añadir piezas ni preguntas que no pidió.
- De Omarchy no se toman ni los atajos ni los valores por defecto. Sus mecanismos solo entran uno a uno y con aprobación.
- Los atajos de Hyprland son los de su sistema anterior, Willwitcher-OS Endgame. Lo nuevo solo ocupa teclas que estaban libres.
- Al agente local se le habla en inglés. Para programar, el usuario usa Claude y no el agente local.

## Herramientas

- `tools/sync-design.py <exportación>`: regenera `docs/DESIGN.md` desde la exportación en Markdown del documento de diseño y comprueba los enlaces del README y del propio documento.
- `tools/screenshots.sh [nombre ...]`: rehace las capturas de `docs/screenshots/` desde el mockup, con Chrome o Chromium sin ventana.

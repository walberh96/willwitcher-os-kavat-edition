# Willwitcher-OS Kavat Edition

Distribución personal basada en Arch Linux. Está en **fase 1**: hay un perfil de archiso en `iso/`, un primer paquete en `packages/` y una ISO que arranca en una máquina virtual hasta un Hyprland y un Noctalia sin modificar. Lo demás del diseño no está implementado. Se compila en el PC de escritorio. Ese PC lleva CachyOS, instalado solo para este proyecto: la ISO usa un `pacman.conf` propio con solo los repos de Arch y `[kavat]`, y los paquetes se compilan en un chroot limpio de Arch, nunca con el `makepkg` del anfitrión.

## Idiomas

- Con el usuario se habla en español.
- `README.md` y el mockup van en inglés. `docs/DESIGN.md`, `docs/PENDING.md` y `reports/` van en español.

## Dónde vive el diseño

Todo el diseño vive en el repo, para que llegue igual a cualquier equipo. Son tres sitios, y los dos primeros deben decir lo mismo:

1. **`docs/DESIGN.md`.** El documento de diseño. Es la fuente y se edita directamente; al editarlo se pone al día la fecha de su cabecera.
2. **`docs/mockup/index.html`.** El mockup interactivo de la interfaz, en un solo HTML. Cuando el usuario pide un «diseño propuesto» quiere verlo dibujado ahí, no descrito en texto.
3. **`docs/PENDING.md`.** El documento «Decisiones pendientes»: lo que espera una decisión del usuario.

El usuario llama «artifact» tanto a los dos documentos como al mockup. Hasta el 11 de noviembre de 2026 los dos documentos vivían en claude.ai y `docs/DESIGN.md` era una exportación; esas copias de claude.ai ya no se usan ni se actualizan.

Los dos diagramas del documento de diseño son imágenes en `docs/img/` y no tienen fuente en el repo: si cambian, se redibujan.

## Cómo se cambia el diseño

1. Lo que el usuario tiene que decidir va a `docs/PENDING.md`, no al documento de diseño. Cada punto lleva un número (D1 a D32 ya se usaron; se sigue en D33), una recomendación y qué pasa si dice que no.
2. Antes de trabajar, se mira ese archivo: el usuario contesta por chat o escribiendo en él.
3. Lo aprobado pasa a `docs/DESIGN.md`, al mockup y, si toca, al README, y sale de pendientes. El documento y el mockup cambian en el mismo commit: no se deja uno para después.
4. Lo rechazado se borra de pendientes y no se vuelve a preguntar. Si el usuario aprueba una parte y dice «lo otro no», ese no cubre todo lo demás que estuviera sobre la mesa.
5. Tras tocar el mockup se rehacen las capturas afectadas con `tools/screenshots.sh <nombre>`.
6. Se hace commit en `main` después de cada cambio de diseño, sin preguntar. El push no va incluido: se hace cuando el usuario lo pide.

## Criterios del usuario

- No complicar: no añadir piezas ni preguntas que no pidió.
- De Omarchy no se toman ni los atajos ni los valores por defecto. Sus mecanismos solo entran uno a uno y con aprobación.
- Los atajos de Hyprland son los de su sistema anterior, Willwitcher-OS Endgame. Lo nuevo solo ocupa teclas que estaban libres.
- Al agente local se le habla en inglés. Para programar, el usuario usa Claude y no el agente local.

## Herramientas

- `tools/check-links.py`: comprueba que los enlaces del README, de `docs/DESIGN.md` y de `docs/PENDING.md` apuntan a archivos que existen. Se pasa tras editar cualquiera de los tres.
- `tools/make-video.cjs [segundos]`: rehace `docs/video/kavat-tour.mp4`, el recorrido narrado por Kavat, desde `docs/video/index.html`, que usa el mockup como imagen. La voz es Supertonic-3, el modelo que el diseño da a Kavat. Pide `puppeteer-core`, `ffmpeg-static` y `supertonic` instalados fuera del repo; el uso está en la cabecera del script. El vídeo va en inglés y presenta el sistema como terminado, por decisión del usuario; el README sigue diciendo en qué fase está de verdad.
- `tools/screenshots.sh [nombre ...]`: rehace las capturas de `docs/screenshots/` desde el mockup, con Chrome o Chromium sin ventana.

## Compilar y probar

Tras tocar `iso/` o `packages/` se pasan los tres, en este orden. Los dos primeros piden sudo; la prueba no.

- `tools/build-repo.sh [paquete ...]`: compila los paquetes de `packages/` en un chroot limpio de Arch y los reúne en `out/repo/`, el repo `[kavat]` local. Pide `devtools`.
- `tools/build-iso.sh`: construye la ISO desde `iso/` y la deja en `out/`. Pide `archiso`.
- `tests/boot.py [iso]`: arranca la ISO en QEMU, comprueba desde dentro que llega al escritorio y deja una captura en `out/test/desktop.png`. Es la puerta de la fase 1. Pide `qemu-desktop` y `edk2-ovmf`.

Los repos de Arch se usan tal como estaban el día que dice `iso/arch-snapshot`; cambiar esa fecha es actualizar el sistema. `out/`, `work/` y `vendor/` no entran en git: son resultados, trabajo temporal y descargas.

La configuración de Hyprland es Lua y su API cambia: antes de usar una función nueva se comprueba en la máquina virtual, no se supone.

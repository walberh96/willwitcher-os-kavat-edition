# Willwitcher-OS Kavat Edition — Decisiones pendientes

Actualizado el 12 de noviembre de 2026 · [Documento de diseño](DESIGN.md) · [Mockup interactivo](mockup/index.html)

## Cómo funciona

Aquí está solo lo que espera tu decisión. Todo lo demás vive en el [documento de diseño](DESIGN.md) y en el mockup.

- **Si apruebas algo**, pasa al documento de diseño y al mockup, y sale de aquí.
- **Si lo rechazas**, se borra de aquí y no se vuelve a preguntar.
- **Para contestar**, basta el número por chat («D22 sí, D31 no») o escribir la respuesta junto a la decisión en este archivo.

## Resumen

Hay 10 decisiones pendientes, de D22 a D31, del 12 de noviembre de 2026. Salen de repasar los paquetes de Willwitcher-OS Endgame: `system_packages.nix`, `user_packages.nix` y lo que activaba `configuration.nix`, en la rama `main` del 3 de diciembre de 2025. Son listas de programas, así que no están dibujadas en el mockup.

| # | Qué es | Recomendación |
| --- | --- | --- |
| D22 | Utilidades de terminal en el sistema base | Sí |
| D23 | Compresión, medios y NTFS en el sistema base | Sí |
| D24 | Teclado Keychron: qmk y via | Sí |
| D25 | Auriculares: headsetcontrol | Sí, si sigues con esos auriculares |
| D26 | Rust en Development | Sí |
| D27 | Marksman y Deno | Marksman sí; Deno solo si lo usas |
| D28 | Signal, LibreOffice, pandoc y TeX Live como casillas del instalador | Sí, sin marcar de serie |
| D29 | pass | Solo si lo sigues usando |
| D30 | Papirus como tema de iconos | Sí |
| D31 | Lutris, gamemode y gemini-cli | No a los tres |

D21, elegir el software en el instalador, quedó aprobada el 12 de noviembre de 2026 y ya está en el documento de diseño.

## Lo que no hay que decidir

**Ya está en Kavat Edition**, igual o con otro nombre: git, gh, zsh, Hyprland, hyprlock, hyprpicker, PipeWire, NetworkManager, Bluetooth, polkit, el llavero de GNOME, el portal de Hyprland, Steam, Vesktop, Chromium (como `kavat-browser`), Neovim, Yazi, ripdrag, lazygit, nb (como `kavat-notes`), fastfetch (como `kavat-fetch`), btop (como `kavat-top`), mpv, zathura, mpvpaper, cliphist, slurp, udiskie (con udisks2) y gnupg (con gpg-agent).

**Lo sustituye una pieza de Kavat Edition**, así que no se trae:

| De Endgame | Lo sustituye |
| --- | --- |
| waybar, rofi, swaynotificationcenter, wlogout | `kavat-shell` |
| hyprpaper, swww, waypaper | Los fondos del shell y mpvpaper |
| hyprshot, gpu-screen-recorder y su ventana | Capturas y grabador integrados en el shell |
| wezterm | Kitty |
| Thunar, xarchiver | Yazi |
| qimgv | imv |
| blueman, el applet de NetworkManager | La página Network & Bluetooth |
| pwvucontrol | La página Sound |
| nwg-look | El sistema de temas |
| seahorse | El llavero, en la página Privacy & network |
| stow | El historial y la sincronización de la configuración |
| GDM, systemd-boot | LightDM y GRUB |

## D22 · Utilidades de terminal en el sistema base

zoxide, lsd, ripgrep, fd, fzf, bat y jq. El mockup ya dibuja en la página Shell los alias a lsd, ripgrep y fd y el plugin de zoxide, pero el documento de diseño no los lista como paquetes.

- **Recomendación**: sí. Son pequeñas y la configuración de zsh ya cuenta con ellas.
- **Si dices que no**: se quitan los alias de la página Shell y se instalan a mano.

## D23 · Compresión, medios y NTFS en el sistema base

7zip y unar para los comprimidos, ffmpeg para vídeo y audio, y soporte de NTFS para discos de Windows. Yazi los usa para abrir comprimidos como carpetas y para las vistas previas; los discos extraíbles, para los USB con NTFS. De Endgame quedarían fuera ripgrep-all y v4l-utils.

- **Recomendación**: sí.
- **Si dices que no**: Yazi no abre todos los comprimidos ni previsualiza vídeo, y un USB con NTFS no se monta.

## D24 · Teclado Keychron

qmk y via, con las reglas de udev de Keychron, para configurar y flashear el teclado.

- **Recomendación**: sí. Es tu teclado y el sistema es solo para ti.
- **Si dices que no**: el teclado funciona, pero se configura desde otro equipo o instalándolo a mano.

## D25 · Auriculares

headsetcontrol con su regla de udev, para ver la batería y ajustar el micrófono de los auriculares inalámbricos por USB.

- **Recomendación**: sí, si sigues con esos auriculares; la batería iría a la página Network & Bluetooth junto a los demás dispositivos.
- **Si dices que no**: los auriculares suenan igual, sin batería a la vista.

## D26 · Rust en Development

rustup con rustc, cargo, rustfmt, clippy y rust-analyzer, más lldb, gcc y pkg-config, como grupo «Rust» de la categoría Development.

- **Recomendación**: sí. `kav` y `kavatd` se escriben en Rust.
- **Si dices que no**: Development se queda con lo de lanequarters y Rust se instala a mano.

## D27 · Marksman y Deno

Marksman es el servidor de lenguaje de Markdown; Deno, un runtime de JavaScript.

- **Recomendación**: Marksman sí, como parte del paquete de Markdown de `kavat-nvim`, porque las notas se editan en Neovim. Deno solo si lo usas: lanequarters no lo pide.
- **Si dices que no**: Markdown en Neovim se queda sin autocompletado de enlaces.

## D28 · Apps de escritorio como casillas del instalador

Signal en Comunicación, y LibreOffice, pandoc y TeX Live en Oficina. No entran en el sistema base: son casillas del paso Software.

- **Recomendación**: sí, sin marcar de serie. TeX Live es grande, así que conviene marcarlo solo si escribes en LaTeX.
- **Si dices que no**: no salen en la lista, aunque el buscador del instalador las sigue encontrando.

## D29 · pass

El gestor de contraseñas de terminal, que guarda cada una cifrada con GPG.

- **Recomendación**: solo si lo sigues usando. El llavero ya guarda las del navegador y los tokens de git.
- **Si dices que no**: no se instala.

## D30 · Papirus como tema de iconos

El documento de diseño no nombra un tema de iconos. Propuesta: Papirus, el de Endgame, con las carpetas en el color de acento.

- **Recomendación**: sí.
- **Si dices que no**: hay que elegir otro; sin tema de iconos, las apps GTK y Qt salen sin iconos coherentes.

## D31 · Lutris, gamemode y gemini-cli

Los tres estaban en Endgame y propongo no traerlos.

- **Lutris**: es una tienda fuera de Steam, como Heroic, que ya rechazaste.
- **gamemode**: el diseño descartó los demonios de rendimiento; el modo Gamer ya pone la CPU en máximo rendimiento.
- **gemini-cli**: para programar usas Claude.
- **Recomendación**: no a los tres.
- **Si dices que sí a alguno**: Lutris iría a la categoría de juegos del instalador, gamemode se activaría con el modo Gamer y gemini-cli sería una casilla de Development.

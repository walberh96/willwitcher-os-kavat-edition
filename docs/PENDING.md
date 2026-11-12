# Willwitcher-OS Kavat Edition — Decisiones pendientes

Actualizado el 12 de noviembre de 2026 · [Documento de diseño](DESIGN.md) · [Mockup interactivo](mockup/index.html)

## Cómo funciona

Aquí está solo lo que espera tu decisión. Todo lo demás vive en el [documento de diseño](DESIGN.md) y en el mockup.

- **Si apruebas algo**, pasa al documento de diseño y al mockup, y sale de aquí.
- **Si lo rechazas**, se borra de aquí y no se vuelve a preguntar.
- **Para contestar**, basta el número por chat («D12 sí, D19 no») o escribir la respuesta junto a la decisión en este archivo.

## Resumen

Hay 11 decisiones pendientes, todas del 12 de noviembre de 2026. D10 y D11 son ideas para mejorar las dos piezas que acabas de añadir, `kavat-fetch` y `kavat-top`, que ya están en el documento de diseño. De D12 a D20 son huecos que encontré al repasar el sistema: componentes que el diseño no nombra.

| # | Qué es | Recomendación |
| --- | --- | --- |
| D10 | Tres ideas para la información del sistema | Sí a las tres |
| D11 | Cuatro ideas para el monitor del sistema | Sí a las cuatro; D11.2 es la más cara |
| D12 | Página de red y Bluetooth en el panel | Sí |
| D13 | Llavero y llaves SSH y GPG | Sí |
| D14 | Apps por defecto para imágenes, vídeo, PDF y comprimidos | Sí |
| D15 | Juegos: MangoHud, gamescope y tiendas fuera de Steam | Sí a MangoHud y gamescope; Heroic solo si lo usas |
| D16 | Discos extraíbles | Sí |
| D17 | Firmware dentro de `kav update` | Sí |
| D18 | Calculadora y conversiones en la paleta | Sí |
| D19 | Impresión y escáner | Solo si tienes impresora |
| D20 | Diálogo de contraseña propio | Sí; sin él lo privilegiado no funciona |

Ninguna está dibujada todavía en el mockup: cada una se dibuja al aprobarla.

## D10 · Ideas para la información del sistema

| # | Idea | Recomendación | Si dices que no |
| --- | --- | --- | --- |
| D10.1 | **Logo como imagen.** En Kitty el gato se pinta como imagen, con el protocolo gráfico de Kitty; fuera de Kitty sale el de caracteres | Sí | Solo el logo de caracteres |
| D10.2 | **Línea de salud.** Una línea con las actualizaciones pendientes y los avisos de `kav doctor`, leída de lo último que `kav` guardó para no frenar | Sí | La salud se mira en `kav` |
| D10.3 | **Saludo al abrir la primera terminal.** Una versión de tres líneas al abrir la primera terminal de la sesión, con un interruptor que viene apagado | Sí, apagado de serie | Solo sale al escribir `kavat fetch` |

## D11 · Ideas para el monitor del sistema

| # | Idea | Recomendación | Si dices que no |
| --- | --- | --- | --- |
| D11.1 | **Pestaña de servicios.** Los servicios de systemd que fallaron y su log, con Kavat explicando el error. Hoy el diseño no tiene dónde ver logs | Sí | Los logs se miran con `journalctl` |
| D11.2 | **Historial de 24 horas.** CPU, memoria, GPU y temperatura del último día, para preguntar «qué pasó a las tres». Pide un recolector siempre encendido, que se detiene en modo Gamer | Sí, aunque es la que más cuesta | El monitor solo enseña el momento actual |
| D11.3 | **Del proceso a sus reglas.** Desde un proceso se salta a su regla del cortafuegos y a cuándo usó el micrófono, la cámara o la pantalla | Sí | Eso se mira en la página Privacy & network |
| D11.4 | **Aviso cuando algo se desboca.** Si un proceso lleva minutos comiéndose la CPU o la memoria, sale un aviso discreto que abre el monitor en ese proceso. Calla en modo Gamer | Sí | No hay aviso; te enteras al notar el equipo lento |

## D12 · Página de red y Bluetooth

El panel tiene el cortafuegos y la VPN de Incognito, pero ninguna página para la red en sí. Propuesta: una página Network con Wi-Fi, cable, VPN guardadas y dispositivos Bluetooth, como mandos y auriculares, con su batería.

- **Recomendación**: sí. Emparejar un mando es de lo primero que se hace en un equipo de juego.
- **Si dices que no**: quedan los interruptores de la barra, y lo demás se hace con `nmtui` y `bluetoothctl`.

## D13 · Llavero y llaves

El diseño no dice dónde se guardan los secretos. Propuesta: un llavero que se abre al iniciar sesión y guarda las contraseñas del navegador y los tokens de git y de `gh`, y un agente para las llaves SSH y GPG, para no teclear su contraseña en cada uso.

- **Recomendación**: sí. Sin llavero, Chromium guarda las contraseñas sin la protección del sistema.
- **Si dices que no**: cada app guarda sus secretos por su cuenta.

## D14 · Apps por defecto para abrir cosas

El diseño habla de «apps por defecto» y de «abrir con», pero solo nombra zathura para PDF. Propuesta: imv para imágenes, mpv para vídeo y audio, zathura para PDF, y los comprimidos se abren dentro de Yazi como carpetas. Todas siguen el tema.

- **Recomendación**: sí.
- **Si dices que no**: se eligen al instalar cada una, y abrir una imagen o un vídeo no hace nada de serie.

## D15 · Juegos: capa de juego y tiendas fuera de Steam

El perfil por juego habla de un «monitor de rendimiento» y de un límite de fotogramas, sin decir con qué. Propuesta: MangoHud para el monitor y el límite, gamescope para fijar resolución y escalado por juego, y Heroic para los juegos de Epic y GOG. Todo en el perfil de juegos del instalador.

- **Recomendación**: sí a MangoHud y gamescope; Heroic solo si tienes juegos en esas tiendas.
- **Si dices que no**: Steam y Proton tal como vienen, y el perfil por juego se queda en el perfil de GPU.

## D16 · Discos extraíbles

El diseño solo nombra el disco externo de las copias. Propuesta: al enchufar un USB se monta solo y sale un aviso con «Abrir en Yazi» y «Expulsar»; la barra enseña los discos montados.

- **Recomendación**: sí.
- **Si dices que no**: se montan a mano.

## D17 · Firmware dentro de `kav update`

`kav` actualiza paquetes, pero no el firmware de la placa, del SSD o de los periféricos. Propuesta: `kav update` consulta fwupd y ofrece el firmware como un paso más, con su aviso de reinicio.

- **Recomendación**: sí.
- **Si dices que no**: el firmware se actualiza a mano con `fwupdmgr`, o no se actualiza.

## D18 · Calculadora y conversiones en la paleta

Propuesta: escribir una cuenta o una conversión de unidades en `Super+D` da el resultado, y Enter lo copia.

- **Recomendación**: sí. Es pequeño y no añade ninguna tecla.
- **Si dices que no**: la paleta solo busca.

## D19 · Impresión y escáner

El diseño no los nombra. Propuesta: un perfil opcional del instalador con CUPS y el escáner, que no se instala si no lo marcas.

- **Recomendación**: solo si tienes impresora; si no, mejor fuera.
- **Si dices que no**: se instala a mano el día que haga falta.

## D20 · Diálogo de contraseña propio

Las acciones privilegiadas del agente, los cambios de arranque y `kav` piden contraseña por polkit, pero el diseño no dice quién dibuja ese diálogo. Propuesta: el shell lo dibuja, con el tema del sistema, diciendo qué programa pide permiso y para qué.

- **Recomendación**: sí. Sin un agente de polkit en la sesión, esas acciones fallan sin preguntar.
- **Si dices que no**: se usa un agente de polkit genérico, que no sigue el tema.

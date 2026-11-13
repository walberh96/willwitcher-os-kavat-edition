# Willwitcher-OS Kavat Edition — Decisiones pendientes

Actualizado el 13 de noviembre de 2026 · [Documento de diseño](DESIGN.md) · [Mockup interactivo](mockup/index.html)

## Cómo funciona

Aquí está solo lo que espera tu decisión. Todo lo demás vive en el [documento de diseño](DESIGN.md) y en el mockup.

- **Si apruebas algo**, pasa al documento de diseño y al mockup, y sale de aquí.
- **Si lo rechazas**, se borra de aquí y no se vuelve a preguntar.
- **Para contestar**, basta el número por chat («D1 sí, D5 no») o escribir la respuesta junto a la decisión en este archivo.

## Resumen

Hay dos decisiones pendientes, las dos del 13 de noviembre de 2026 y salidas al implementar el instalador, en la fase 2:

- **D33**: quién abre el disco cuando está cifrado, y por tanto dónde vive el kernel. Mientras no se decida, el instalador no ofrece el cifrado.
- **D34**: cómo se hace Secure Boot, porque GRUB sin shim no arranca ningún kernel con Secure Boot activo. Mientras no se decida, el instalador deja tus llaves y las firmas listas, pero no lo activa.

La anterior, D32, se resolvió el 12 de noviembre de 2026: los iconos y el cursor se generan desde el tema y ya están en el documento de diseño.

## D33 · Quién abre el disco cifrado

Sin cifrar, el sistema ya funciona como dice el diseño: el kernel vive en `/boot`, dentro del sistema, así que cada snapshot lleva el suyo, y GRUB lo lee directamente. Con el disco cifrado, alguien tiene que abrirlo antes de poder leer ese kernel. Hay dos formas:

| Opción | Cómo funciona | A favor | En contra |
| --- | --- | --- | --- |
| A | **GRUB abre el disco.** La clave va sellada en el TPM2 y GRUB la saca de ahí; el kernel sigue dentro de cada snapshot. Después, el sistema abre el disco otra vez con una llave guardada dentro de su imagen de arranque, que también está cifrada, así que no pregunta dos veces | Es el diseño tal como está: cada snapshot arranca con su propio kernel, con o sin cifrado. Todo lo que hay en el disco, kernel incluido, queda cifrado | El TPM2 en GRUB es reciente y está menos probado que el de systemd; el GRUB de Arch ya lo trae. Si el TPM no abre, la contraseña de recuperación la pide GRUB en texto, sin el tema, y no la pantalla de carga |
| B | **El sistema abre el disco.** El kernel de cada snapshot se copia, sin cifrar, a la partición EFI, y GRUB lo arranca de ahí. Abre el disco systemd, con el TPM2, que es el camino más usado | Desbloqueo por TPM2 probado y algo más rápido. La contraseña de recuperación se pide en la pantalla de carga, con el tema, como dice el diseño | Hace falta una pieza propia que copie los kernels a la partición EFI en cada snapshot y escriba el menú de snapshots, en lugar de grub-btrfs. La partición EFI pasa de 1 a 2 GB. El kernel y la imagen de arranque quedan sin cifrar |

- **Recomendación**: A, porque no cambia nada del diseño ni añade piezas propias, y deja cifrado todo el disco. Si en tu equipo el TPM2 de GRUB diera problemas, B queda como plan B.
- **Si dices que no a las dos**, el cifrado se queda sin TPM2: GRUB pide la contraseña al arrancar, una sola vez, y el resto funciona como en A.

Lo que no cambia con ninguna: el cifrado sigue siendo un interruptor del instalador, apagado por defecto, y la contraseña de recuperación existe siempre.

## D34 · Secure Boot con GRUB

Al probarlo en máquina virtual salió una limitación del GRUB oficial: con Secure Boot activo, GRUB verifica todo lo que carga, y solo sabe hacerlo a través de shim, el pequeño cargador firmado por Microsoft que usan Ubuntu y Fedora. Sin shim, GRUB se niega a arrancar cualquier kernel, aunque esté firmado con tus llaves. Los foros de Arch tienen el mismo caso, y quienes lo intentaron acabaron cambiando de gestor de arranque. Hoy el instalador crea tus llaves y firma GRUB y los kernels, pero no activa Secure Boot, porque el equipo dejaría de arrancar.

| Opción | Cómo funciona | A favor | En contra |
| --- | --- | --- | --- |
| A | **Añadir shim.** El firmware arranca shim, firmado por Microsoft; shim arranca GRUB y le da con qué verificar los kernels, que siguen firmados con tus llaves | Se queda GRUB, con su menú y sus snapshots. Es el camino de Ubuntu y Fedora | shim no está en los repos de Arch: entra por la allowlist del AUR, que lo toma ya firmado de Fedora. Hay que comprobar que acepta la versión de GRUB de Arch |
| B | **GRUB verifica con firmas GPG.** Se mete en GRUB una llave GPG, y cada archivo que lee lleva su firma al lado: kernels, imágenes de arranque, el menú y la lista de snapshots | Sin piezas de fuera, y verifica también la imagen de arranque, cosa que Secure Boot solo no hace | Casi nadie lo usa así. Hay que volver a firmar cada vez que algo cambia, también al crear un snapshot, y un fallo de firma deja el equipo sin arrancar |
| C | **Cambiar de gestor de arranque**, a systemd-boot o a Limine | Secure Boot funciona sin rodeos | Cambia una decisión tomada, GRUB con menú propio, y hay que rehacer el menú, los snapshots arrancables y el modo seguro sobre el nuevo |
| D | **Dejarlo apagado por ahora.** Las llaves y las firmas quedan listas para cuando haya solución | Nada que hacer | Sin Secure Boot. Y el desbloqueo por TPM2 de D33 protege menos, porque se apoya en que el arranque esté firmado |

- **Recomendación**: A, porque mantiene todo lo decidido y es el camino más usado. Si al probarlo shim rechazara el GRUB de Arch, quedaría D mientras tanto.
- **Si dices que no a todas**, queda D.

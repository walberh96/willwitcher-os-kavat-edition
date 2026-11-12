# Willwitcher-OS Kavat Edition — Decisiones pendientes

Actualizado el 12 de noviembre de 2026 · [Documento de diseño](DESIGN.md) · [Mockup interactivo](mockup/index.html)

## Cómo funciona

Aquí está solo lo que espera tu decisión. Todo lo demás vive en el [documento de diseño](DESIGN.md) y en el mockup.

- **Si apruebas algo**, pasa al documento de diseño y al mockup, y sale de aquí.
- **Si lo rechazas**, se borra de aquí y no se vuelve a preguntar.
- **Para contestar**, basta el número por chat («D1 sí, D5 no») o escribir la respuesta junto a la decisión en este archivo.

## Resumen

Hay una decisión pendiente, D21, del 12 de noviembre de 2026. Está dibujada en el mockup: escena Installer con `istep=software`, captura `docs/screenshots/installer-software.png`.

Las anteriores, de D10 a D20, quedaron aprobadas el 12 de noviembre de 2026 y ya están en el documento de diseño; de D15 se rechazó Heroic.

## D21 · Elegir el software en el instalador

Hoy el paso 5 del instalador, «Perfil», ofrece tres conjuntos cerrados (desarrollo, creación, juegos). Propuesta: ese paso pasa a llamarse «Software» y deja marcar programas uno a uno. El instalador sigue teniendo 7 pantallas.

| # | Qué se propone | Recomendación | Si dices que no |
| --- | --- | --- | --- |
| D21.1 | **El paso Software.** Categorías a la izquierda (navegadores, desarrollo, creación, juegos, comunicación, multimedia, oficina) y casillas por programa, como Firefox u OBS. Un buscador añade cualquier paquete de Arch, de `[kavat]` o de la allowlist del AUR. Lo instala `kav` con sus reglas de siempre, y lo marcado se guarda en `kavat-install.toml` para repetirlo al reinstalar | Sí | Se quedan los tres conjuntos cerrados y lo demás se instala después con `kav` |
| D21.2 | **La categoría Development**, con lo que pide lanequarters. La lista está debajo | Sí | Development se queda en git, docker y mise |
| D21.3 | **Sin red, a la cola.** La ISO sigue instalando el sistema base sin red. Lo marcado se descarga durante la instalación si hay red; si no la hay, queda en cola y `kav` lo instala en el primer arranque con red | Sí | Sin red, el paso Software sale desactivado |
| D21.4 | **Lo que no es un paquete.** vercel, eas-cli y los navegadores de Playwright se instalan con pnpm, no con pacman. El conjunto los instala con pnpm en el primer arranque, y `kav` no los gestiona | Sí | Esos tres se instalan a mano en cada proyecto |

**La lista de Development.** Sale de leer los tres proyectos de lanequarters: web (Next.js, Prisma, Supabase, Stripe, Playwright), móvil (Expo, con pruebas de Maestro) y extensión (WXT).

| Grupo | Programas | Para qué |
| --- | --- | --- |
| Esenciales | git, github-cli, mise, pnpm, docker con compose, claude-code | Los tres proyectos piden Node 22 o superior y pnpm 11; mise pone la versión de Node de cada uno |
| Web | supabase, postgresql (cliente `psql`), stripe-cli, navegadores de Playwright, vercel | `supabase start` levanta la base local en Docker; `stripe listen` recibe los webhooks |
| Móvil | jdk17-openjdk, Android SDK con emulador, android-tools, watchman, maestro, eas-cli | Compilar y probar la app de Android |
| Extensión | firefox, y el propio `kavat-browser` | WXT prueba la extensión en los dos navegadores |

Tres límites:

- **iOS no se compila en Linux.** La app de iOS se sigue compilando en la Mac o con EAS en la nube.
- **Seis vienen del AUR**: claude-code, supabase, stripe-cli, el SDK de Android, watchman y maestro. Cada uno necesita su entrada en la allowlist, y los que se compilan alargan la instalación.
- **Sin comprobar**: no he verificado en qué repositorio está hoy cada paquete; se comprueba al escribir la allowlist.

# Fauste Player — Guía del usuario

<div class="ai-notice" role="note">Esta guía se ha traducido con IA a partir de la guía en inglés (v0.13.1-15-gfcd6eac6, commit fcd6eac) y puede contener errores. Ante cualquier diferencia, vale la versión en inglés. <a href="../index.html" hreflang="en">Leer el original en inglés</a></div>

Fauste Player es una aplicación de emisión (playout) para estudios de radio.
Ejecuta varios **players** independientes, uno al lado del otro. Cada player
tiene sus propias playlists y su propio transporte. Los players pueden
encadenar pistas con mezclas solapadas, preescuchar en una salida **CUE**
aparte y sobrevivir a fallos sin poner nada al aire por sí solos.

> **Desarrollado con ayuda de IA y ofrecido tal cual.** Fauste Player, esta
> guía y sus traducciones se han desarrollado con ayuda sustancial de
> herramientas de IA. Se ofrece sin ninguna garantía y lo usas bajo tu propio
> riesgo: consulta el
> [aviso legal](https://github.com/sanchezfauste/fauste-player#disclaimer).

![Pantalla principal](../../images/main-screen.png)

| Guía | Qué trata |
|---|---|
| [Primeros pasos](getting-started.md) | Instalación, primer arranque, añadir música, reproducir |
| [Players](players.md) | La columna del player: transporte, modos, cuenta atrás, forma de onda, CUE |
| [Playlists](playlists.md) | Pestañas, la tabla de pistas, arrastrar y soltar, el menú contextual, archivos M3U/PLS |
| [Cartuchera](cartwall.md) | Cartuchos instantáneos: disparo, páginas, bucles, cartuchos exclusivos, configuración |
| [Marcadores y mezcla](markers-and-mixing.md) | Cue-in/cue-out, el punto MIX, intro y outro, análisis automático |
| [Configuración](settings.md) | Salidas de audio, players, análisis, playlists, MIDI, control remoto |
| [Normalización de la sonoridad](loudness.md) | Todas las pistas y cartuchos con la misma sonoridad mediante una ganancia fija por pista: el distintivo de ganancia, excluir y ajustar una pista, salidas bit-perfect |
| [Salida bit-perfect](bit-perfect.md) | Un modo especial: sin fundidos encadenados, volumen al 100 %. Dispositivos exclusivos que siguen la frecuencia de cada archivo, el distintivo BP, salida DSD, comprobar una cadena |
| [Teclado](keyboard.md) | Atajos |
| [Superficies de control MIDI](midi.md) | Botones, faders y luces en controladores MIDI |
| [Control remoto](remote-control.md) | Manejar Fauste Player por la red: API HTTP, seguridad |
| [Datos y copias de seguridad](data-and-backups.md) | Dónde se guardan los archivos, guardado automático, recuperación tras un fallo, modo portátil |
| [Solución de problemas](troubleshooting.md) | Sin sonido, dispositivo perdido, archivos marcados como no disponibles, registros |

La interfaz está disponible en inglés, español, catalán, neerlandés, francés,
gallego, alemán, italiano, euskera, polaco y portugués. Sigue el idioma del
sistema operativo salvo que elijas uno en Configuración. La interfaz en todos
los idiomas, incluidos el inglés y el español, se ha generado con IA y puede
contener errores. Las correcciones son bienvenidas como issues o pull
requests.

La guía también está disponible en otros idiomas, traducida con IA
a partir de la guía en inglés: en la [guía en línea](https://sanchezfauste.com/fauste-player/guide/),
elige uno con el botón del globo de la barra superior.
Una página traducida lo indica arriba y enlaza con la página en
inglés, que es la de referencia cuando las dos no coinciden.

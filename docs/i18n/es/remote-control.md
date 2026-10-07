# Control remoto

Fauste Player se puede consultar y manejar por la red a través de una API
HTTP, con actualizaciones en directo, y a través de OSC. Pueden usarlas una
página web, una aplicación de móvil, la automatización de una emisora o una
superficie de control. Está **desactivado** hasta que lo activas, y al
principio solo responde en este ordenador.

## Activarlo {#turning-it-on}

![Configuración, Remoto: la API HTTP activada y escuchando en este ordenador, y OSC desactivado](../../images/guide/settings-remote.png)

Abre **Configuración → Remoto** y marca **Permitir el control remoto por
HTTP** (o **Permitir el control por OSC**). La línea de debajo de cada
interruptor dice si el servidor está escuchando, y dónde, o por qué no ha
arrancado. Los cambios se aplican al momento; no hace falta reiniciar. Un
campo de texto (una dirección, el token, una lista) se aplica al salir de
él, al abrir otra sección o al cerrar Configuración; un valor que todavía no
es válido mantiene el que está en uso, y Esc cancela lo que has escrito.

También puedes editar `config.json` con Fauste Player cerrado (consulta
[Datos y copias de seguridad](data-and-backups.md) para saber dónde está).
Dentro del objeto `"config"`, pon `remote.http.enabled` a `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Escucha en `http://127.0.0.1:7380`. Ponerlo en marcha nunca reproduce nada;
solo actúan las peticiones.

## Escuchar en la red del estudio {#listening-on-the-studio-network}

Para llegar a él desde otros ordenadores, pon `bind` a `0.0.0.0` (o a una
de las direcciones de este ordenador) y fija un **token** de al menos 16
caracteres. En Configuración → Remoto, **Generar** crea un token largo al
azar. Está oculto hasta que pulsas **Mostrar**, y **Copiar** lo pone en el
portapapeles para el cliente. Sin token, el servidor se niega a arrancar, y
el registro explica por qué.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Los clientes envían el token como `Authorization: Bearer <token>`. La API no
va cifrada. Mantenla en una red de estudio de confianza, o ponla detrás de
un proxy inverso con HTTPS.

## Páginas web {#web-pages}

Una página web servida desde otra dirección solo puede usar la API si su
origen (por ejemplo `https://studio.example`) figura en `cors_origins`. Las
peticiones de otras páginas se rechazan, incluso en este ordenador, para que
una página que tengas abierta por casualidad no pueda manejar el
reproductor. `"*"` (cualquier origen) solo se acepta junto con un token.

## Qué puede hacer un cliente {#what-a-client-can-do}

Un cliente puede:

- consultar los players, las playlists, las pistas (con carátula y forma de
  onda) y la cartuchera;
- reproducir, pausar, detener, hacer un fundido, reiniciar y retroceder;
- elegir la siguiente entrada (también la que suena: se reproduce una vez
  más), preescuchar y saltar a una posición (en un player detenido, un salto
  elige dónde empieza Play la siguiente entrada, y un salto antes de su
  cue-in empieza en el cue-in);
- fijar volúmenes, modos, stop al final de la pista actual, y las marcas de
  repetir y de parar después de una entrada;
- disparar, detener y preescuchar cartuchos, y cambiar la página de
  cartuchos que se ve;
- editar: crear, renombrar y eliminar playlists; añadir una pista que ya
  está cargada, y quitar, mover o duplicar entradas; crear, renombrar,
  redimensionar y eliminar páginas de cartuchos, y configurar un cartucho
  con una pista cargada; fijar o restablecer marcadores.

Quitar lo que suena se rechaza, igual que en pantalla. Los archivos que aún
no están cargados no se pueden añadir a distancia: están en este ordenador,
así que añádelos antes aquí.

Un botón que en pantalla aparece en gris también se rechaza a distancia. La
referencia completa está en
[la documentación técnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md)
(en inglés).

Pruébalo desde un terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Actualizaciones en directo {#live-updates}

Un cliente puede seguir los cambios a medida que se producen en lugar de
preguntar una y otra vez. `GET /api/v1/events` es un flujo de eventos:
primero el estado completo, luego cada cambio de un player, una playlist,
una pista o la cartuchera, y los tiempos de lo que suena varias veces por
segundo.

    curl -sN http://127.0.0.1:7380/api/v1/events

Una página web usa `EventSource`. Ahí los navegadores no pueden enviar el
token como cabecera, así que va en la dirección:
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC es el protocolo habitual de las superficies de control, las mesas de
iluminación y el software de control de espectáculos. Actívalo con
`remote.osc.enabled`. Escucha en el puerto UDP 7381 de este ordenador. Para
aceptar paquetes de otros ordenadores, pon `remote.osc.bind` a `0.0.0.0` y
lista sus direcciones o subredes en `remote.osc.allowed_sources` (por
ejemplo `"192.168.1.0/24"`). OSC no tiene contraseña, así que mantenlo en
una red de estudio de confianza.

Los players se numeran 1, 2, 3… tal como aparecen en pantalla. Los cartuchos
se numeran dentro de la página que se ve.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Una superficie que quiere mostrar el estado (luces, nombres, cuentas atrás)
se suscribe, y entonces recibe cada valor una vez y después solo lo que
cambia. Cuando desaparece un player o un botón de cartucho (menos players,
una página más pequeña), sus direcciones reciben una vez un valor vacío,
para que la superficie las borre. Debe volver a suscribirse antes de un
minuto (`subscription_ttl_secs`) para seguir recibiendo:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Un suscriptor puede indicar cualquier puerto de su propia dirección, y se
mantienen hasta `max_subscribers`. Por tanto, cualquiera que pueda enviar
también puede suscribirse. Es un motivo más para mantener OSC en una red de
confianza.

`oscsend` y `oscdump` vienen con liblo (`liblo-tools` en Debian y Ubuntu).
La lista completa de direcciones está en
[la documentación técnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc)
(en inglés).

## Todos los ajustes {#all-settings}

| Ajuste | Por defecto | Significado |
|---|---|---|
| `remote.http.enabled` | `false` | Activa la API |
| `remote.http.bind` | `127.0.0.1` | Dirección en la que escuchar (una dirección IP) |
| `remote.http.port` | `7380` | Puerto (1024–65535) |
| `remote.http.token` | vacío | Obligatorio fuera de este ordenador; al menos 16 caracteres |
| `remote.http.cors_origins` | ninguno | Orígenes web que pueden llamar a la API |
| `remote.http.request_timeout_ms` | `10000` | Lo máximo que puede durar una petición |
| `remote.http.max_body_bytes` | `65536` | Cuerpo de petición más grande |
| `remote.http.max_event_clients` | `16` | Flujos de eventos en directo a la vez |
| `remote.osc.enabled` | `false` | Activa OSC |
| `remote.osc.bind` | `127.0.0.1` | Dirección en la que escuchar |
| `remote.osc.port` | `7381` | Puerto UDP (1024–65535) |
| `remote.osc.allowed_sources` | este ordenador | Direcciones o subredes cuyos paquetes se aceptan |
| `remote.osc.max_subscribers` | `16` | Suscriptores a la vez |
| `remote.osc.subscription_ttl_secs` | `60` | Una suscripción que no se renueva en este tiempo termina |
| `remote.events.position_interval_ms` | `250` | Cada cuánto se publican los tiempos mientras suena algo |

Los valores fuera de rango se corrigen al cargar el archivo, y la corrección
queda en el registro.

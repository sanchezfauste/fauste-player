# Control remoto

Fauste Player pódese ler e manexar pola rede a través dunha API HTTP, con
actualizacións en directo, e a través de OSC. Poden usalo unha páxina web,
unha aplicación de teléfono, a automatización dunha emisora ou unha
superficie de control. Está **desactivado** ata que o actives, e no principio
só responde neste computador.

## Activalo {#turning-it-on}

![Configuración, Remoto: a API HTTP activada e a escoitar neste computador, e OSC desactivado](../../images/guide/settings-remote.png)

Abre **Configuración → Remoto** e marca **Permitir o control remoto por
HTTP** (ou **Permitir o control por OSC**). A liña baixo cada interruptor di
se o servidor está a escoitar, e onde, ou por que non arrancou. Os cambios
aplícanse ao instante; non fai falta reiniciar. Un campo de texto (un
enderezo, o testemuño, unha lista) aplícase cando sales del, abres outra
sección ou pechas Configuración; un valor que aínda non é válido conserva o
que está en uso, e Esc cancela o que escribiches.

Tamén podes editar `config.json` mentres Fauste Player está pechado (consulta
[Datos e copias de seguranza](data-and-backups.md) para ver onde está).
Dentro do obxecto `"config"`, pon `remote.http.enabled` a `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Escoita en `http://127.0.0.1:7380`. Iniciar o servidor nunca reproduce nada;
só actúan as peticións.

## Escoitar na rede do estudo {#listening-on-the-studio-network}

Para acceder desde outros computadores, pon `bind` a `0.0.0.0` (ou a un dos
enderezos deste computador) e define un **testemuño** de polo menos 16
caracteres. En Configuración → Remoto, **Xerar** crea un testemuño aleatorio
longo. Está oculto ata que premes **Mostrar**, e **Copiar** ponio no
portapapeis para o cliente.
Sen testemuño o servidor négase a arrancar, e o rexistro di por que.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Os clientes envían o testemuño como `Authorization: Bearer <token>`. A API
non está cifrada. Mantén a API nunha rede de estudo de confianza, ou ponla detrás
dun proxy inverso con HTTPS.

## Páxinas web {#web-pages}

Unha páxina web servida desde outro enderezo só pode usar a API se a súa
orixe (por exemplo `https://studio.example`) está listada en `cors_origins`.
As peticións doutras páxinas rexéitanse, mesmo neste computador, para que
unha páxina que teñas aberta sen máis non poida manexar o reprodutor. `"*"`
(calquera orixe) só se acepta xunto cun testemuño.

## Que pode facer un cliente {#what-a-client-can-do}

Un cliente pode:

- ler os reprodutores, as listas, as pistas (con portada e forma de onda) e a
  cartucheira;
- reproducir, pausar, parar, facer fundidos, reiniciar e volver atrás;
- escoller a entrada seguinte (tamén a entrada en antena: reprodúcese unha
  vez máis), preescoitar e buscar (nun reprodutor detido, buscar escolle onde
  Play inicia a entrada seguinte, e buscar antes do seu cue-in inicia no
  cue-in);
- axustar volumes, modos, stop ao final da actual, e as marcas de repetir e
  parar despois dunha entrada;
- disparar, parar e preescoitar cartuchos, e cambiar a páxina de cartuchos
  que se ve;
- editar: crear, renomear e eliminar listas; engadir unha pista que xa está
  cargada, e quitar, mover ou duplicar entradas; crear, renomear,
  redimensionar e eliminar páxinas de cartuchos, e configurar un cartucho cunha
  pista cargada; definir ou restablecer marcadores.

Quitar o que está en antena rexéitase, como na pantalla. Os ficheiros que
aínda non están cargados non se poden engadir en remoto: están neste
computador, así que engádeos primeiro aquí.

Un botón atenuado na pantalla tamén se rexeita en remoto. A referencia
completa está na [documentación técnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Próbao desde un terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Actualizacións en directo {#live-updates}

Un cliente pode seguir os cambios a medida que se producen en vez de
preguntar unha e outra vez. `GET /api/v1/events` é un fluxo de eventos: primeiro
o estado completo, despois cada cambio de reprodutor, lista, pista ou
cartucheira, e os tempos do que se reproduce varias veces por segundo.

    curl -sN http://127.0.0.1:7380/api/v1/events

Unha páxina web usa `EventSource`. Os navegadores non poden enviar o
testemuño como cabeceira aí, así que vai no enderezo:
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC é o protocolo habitual das superficies de control, as mesas de iluminación
e o software de control de espectáculos. Actívao con `remote.osc.enabled`.
Escoita no porto UDP 7381 deste computador. Para aceptar paquetes doutros
computadores, pon `remote.osc.bind` a `0.0.0.0` e lista os seus enderezos ou
subredes en `remote.osc.allowed_sources` (por exemplo `"192.168.1.0/24"`).
OSC non ten contrasinal, así que mantén OSC nunha rede de estudo de confianza.

Os reprodutores numéranse 1, 2, 3… como aparecen na pantalla. Os cartuchos
numéranse na páxina que se ve.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Unha superficie que queira mostrar o estado (luces, nomes, contas atrás)
subscríbese, e entón recibe cada valor unha vez e despois só o que cambia.
Cando un reprodutor ou un botón de cartucho desaparece (menos reprodutores,
unha páxina máis pequena), os seus enderezos reciben un valor baleiro unha
vez, para que a superficie os limpe. Debe subscribirse de novo nun minuto
(`subscription_ttl_secs`) para seguir recibindo:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Un subscritor pode nomear calquera porto do seu propio enderezo, e consérvanse
ata `max_subscribers`. Polo tanto, quen estea autorizado a enviar tamén pode
subscribirse. É un motivo máis para manter OSC nunha rede de confianza.

`oscsend` e `oscdump` veñen con liblo (`liblo-tools` en Debian e Ubuntu). A
lista completa de enderezos está na
[documentación técnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Todos os axustes {#all-settings}

| Axuste | Predeterminado | Significado |
|---|---|---|
| `remote.http.enabled` | `false` | Activa a API |
| `remote.http.bind` | `127.0.0.1` | Enderezo no que escoitar (un enderezo IP) |
| `remote.http.port` | `7380` | Porto (1024–65535) |
| `remote.http.token` | baleiro | Necesario fóra deste computador; polo menos 16 caracteres |
| `remote.http.cors_origins` | ningunha | Orixes web autorizadas a chamar á API |
| `remote.http.request_timeout_ms` | `10000` | O máximo que pode tardar unha petición |
| `remote.http.max_body_bytes` | `65536` | O corpo de petición máis grande |
| `remote.http.max_event_clients` | `16` | Fluxos de eventos en directo á vez |
| `remote.osc.enabled` | `false` | Activa OSC |
| `remote.osc.bind` | `127.0.0.1` | Enderezo no que escoitar |
| `remote.osc.port` | `7381` | Porto UDP (1024–65535) |
| `remote.osc.allowed_sources` | este computador | Enderezos ou subredes cuxos paquetes se aceptan |
| `remote.osc.max_subscribers` | `16` | Subscritores á vez |
| `remote.osc.subscription_ttl_secs` | `60` | Unha subscrición que non se renova neste tempo remata |
| `remote.events.position_interval_ms` | `250` | Con que frecuencia se publican os tempos mentres se reproduce |

Os valores fóra de rango corríxense cando se carga o ficheiro, e a corrección
queda no rexistro.

# Controlo remoto

O Fauste Player pode ser lido e operado pela rede através de uma API HTTP,
com atualizações em direto, e através de OSC. Uma página web, uma aplicação
de telemóvel, a automação de uma estação ou uma superfície de controlo podem
usá-los. Está **desativado** até o ativar e, no início, só responde neste
computador.

## Ativar {#turning-it-on}

![Definições, Remoto: a API HTTP ativada e à escuta neste computador, e o OSC desativado](../../images/guide/settings-remote.png)

Abra **Definições → Remoto** e assinale **Permitir o controlo remoto por
HTTP** (ou **Permitir o controlo por OSC**). A linha por baixo de cada
interruptor diz se o servidor está à escuta, e onde, ou porque não arrancou.
As alterações aplicam-se de imediato; não é preciso reiniciar. Um campo de
texto (um endereço, o token, uma lista) aplica-se quando o deixa, abre outra
secção ou fecha as Definições; um valor que ainda não é válido mantém o que
está em uso, e Esc cancela o que escreveu.

Também pode editar o `config.json` com o Fauste Player fechado (ver
[Dados e cópias de segurança](data-and-backups.md) para saber onde está).
Dentro do objeto `"config"`, defina `remote.http.enabled` como `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Fica à escuta em `http://127.0.0.1:7380`. Iniciá-lo nunca toca nada; só os
pedidos atuam.

## Escutar na rede do estúdio {#listening-on-the-studio-network}

Para lhe aceder a partir de outros computadores, defina `bind` como
`0.0.0.0` (ou um dos endereços deste computador) e defina um **token** de
pelo menos 16 caracteres. Em Definições → Remoto, **Gerar** cria um token
aleatório longo. Fica oculto até premir **Mostrar**, e **Copiar** põe-no na
área de transferência para o cliente. Sem um token, o servidor recusa-se a
arrancar, e o registo diz porquê.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Os clientes enviam o token como `Authorization: Bearer <token>`. A API não
é cifrada. Mantenha-a numa rede de estúdio de confiança, ou ponha-a atrás de
um proxy inverso com HTTPS.

## Páginas web {#web-pages}

Uma página web servida a partir de outro endereço só pode usar a API se a sua
origem (por exemplo, `https://studio.example`) estiver listada em
`cors_origins`. Os pedidos de outras páginas são recusados, mesmo neste
computador, para que uma página que por acaso tenha aberta não possa
controlar o leitor. `"*"` (qualquer origem) só é aceite juntamente com um
token.

## O que um cliente pode fazer {#what-a-client-can-do}

Um cliente pode:

- ler os leitores, as listas, as faixas (com capa e forma de onda) e a
  cartucheira;
- tocar, pausar, parar, fazer fade, reiniciar e recuar;
- escolher a entrada seguinte (também a entrada no ar: toca mais uma vez),
  pré-escutar e procurar (num leitor parado, uma procura escolhe onde o Play
  inicia a entrada seguinte, e uma procura antes do seu cue-in começa no
  cue-in);
- definir volumes, modos, stop no fim da faixa atual e as marcas de repetir e
  de parar após a faixa de uma entrada;
- disparar, parar e pré-escutar cartuchos, e mudar a página de cartuchos
  visível;
- editar: criar, renomear e eliminar listas; adicionar uma faixa que já está
  carregada e remover, mover ou duplicar entradas; criar, renomear,
  redimensionar e eliminar páginas de cartuchos, e configurar um cartucho com
  uma faixa carregada; definir ou repor marcadores.

Remover o que está no ar é recusado, como no ecrã. Os ficheiros que ainda
não estão carregados não podem ser adicionados remotamente: vivem neste
computador, por isso adicione-os primeiro aqui.

Um botão que está a cinzento no ecrã também é recusado remotamente. A
referência completa está na
[documentação técnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Experimente num terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Atualizações em direto {#live-updates}

Um cliente pode acompanhar as alterações à medida que acontecem em vez de
perguntar uma e outra vez. `GET /api/v1/events` é um fluxo de eventos: primeiro
o estado completo, depois cada alteração de leitor, lista, faixa ou
cartucheira, e os tempos do que está a tocar algumas vezes por segundo.

    curl -sN http://127.0.0.1:7380/api/v1/events

Uma página web usa `EventSource`. Os navegadores não podem enviar aí o token
como cabeçalho, por isso vai no endereço:
`/api/v1/events?token=<token>`.

## OSC {#osc}

O OSC é o protocolo habitual das superfícies de controlo, das mesas de luzes
e do software de controlo de espetáculos. Ative-o com `remote.osc.enabled`.
Fica à escuta na porta UDP 7381 deste computador. Para aceitar pacotes de
outros computadores, defina `remote.osc.bind` como `0.0.0.0` e liste os seus
endereços ou sub-redes em `remote.osc.allowed_sources` (por exemplo,
`"192.168.1.0/24"`). O OSC não tem palavra-passe, por isso mantenha-o numa
rede de estúdio de confiança.

Os leitores são numerados 1, 2, 3… como aparecem no ecrã. Os cartuchos são
numerados na página visível.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Uma superfície que queira mostrar o estado (luzes, nomes, contagens
decrescentes) subscreve, e depois recebe cada valor uma vez e a seguir só o
que muda. Quando um leitor ou um botão de cartucho desaparece (menos
leitores, uma página mais pequena), os seus endereços recebem uma vez um
valor vazio, para que a superfície os limpe. Tem de subscrever de novo num
minuto (`subscription_ttl_secs`) para continuar a receber:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Um subscritor pode indicar qualquer porta do seu próprio endereço, e são
mantidos até `max_subscribers`. Quem pode enviar pode, portanto, também
subscrever. Esta é mais uma razão para manter o OSC numa rede de confiança.

O `oscsend` e o `oscdump` vêm com a liblo (`liblo-tools` no Debian e no
Ubuntu). A lista completa de endereços está na
[documentação técnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Todas as definições {#all-settings}

| Definição | Predefinição | Significado |
|---|---|---|
| `remote.http.enabled` | `false` | Ativar a API |
| `remote.http.bind` | `127.0.0.1` | Endereço onde escutar (um endereço IP) |
| `remote.http.port` | `7380` | Porta (1024–65535) |
| `remote.http.token` | vazio | Obrigatório fora deste computador; pelo menos 16 caracteres |
| `remote.http.cors_origins` | nenhuma | Origens web autorizadas a chamar a API |
| `remote.http.request_timeout_ms` | `10000` | O máximo que um pedido pode demorar |
| `remote.http.max_body_bytes` | `65536` | O maior corpo de pedido |
| `remote.http.max_event_clients` | `16` | Fluxos de eventos em direto em simultâneo |
| `remote.osc.enabled` | `false` | Ativar o OSC |
| `remote.osc.bind` | `127.0.0.1` | Endereço onde escutar |
| `remote.osc.port` | `7381` | Porta UDP (1024–65535) |
| `remote.osc.allowed_sources` | este computador | Endereços ou sub-redes cujos pacotes são aceites |
| `remote.osc.max_subscribers` | `16` | Subscritores em simultâneo |
| `remote.osc.subscription_ttl_secs` | `60` | Uma subscrição não renovada neste tempo termina |
| `remote.events.position_interval_ms` | `250` | Com que frequência os tempos são publicados durante a reprodução |

Os valores fora do intervalo são corrigidos quando o ficheiro é carregado, e
a correção fica registada.

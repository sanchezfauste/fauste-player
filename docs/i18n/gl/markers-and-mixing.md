# Marcadores e mestura

Cada pista ten ata cinco **marcadores**, en segundos:

| Marcador | Significado | Como se define |
|---|---|---|
| Cue in | Onde empeza a reprodución | Automático: xusto antes do primeiro son por riba do limiar de recorte |
| Cue out | Onde remata a pista | Automático: xusto despois do último son por riba do limiar de recorte |
| MIX (inicio do segue) | Onde empeza a pista seguinte no modo continuo | Automático (ver abaixo) |
| Inicio do outro | Onde comeza o final da pista | Automático (ver abaixo) |
| Fin da intro | Final da introdución falada por riba | A man, ou desde unha etiqueta `INTRO` do ficheiro |

Os marcadores definidos a man sempre gañan: unha análise nova nunca os
substitúe.

## Editar marcadores {#editing-markers}

Na forma de onda dun reprodutor, ou na da súa xanela CUE (o mesmo menú e as
mesmas asas; un cambio vese nas dúas ao instante):

- **Clic dereito** onde queiras un marcador e escolle **Poñer o cue-in aquí**,
  **Poñer o fin da intro aquí**, **Poñer o inicio do outro aquí**, **Poñer o
  punto MIX aquí** ou **Poñer o cue-out aquí**. **Volver aos marcadores
  automáticos** quita os marcadores que colocaches, e a pista analízase de
  novo.
- **Mantén Alt** (Option en macOS): aparecen asas nos marcadores. Arrastra
  unha para movela; o tempo móstrase mentres arrastras. Un arrastre nunca
  move a cabeza de reprodución.

O cue-in debe quedar antes do cue-out. Os outros marcadores quedan entre
eles. Os cambios na pista en antena aplícanse ao instante á súa seguinte
transición.

## A etiqueta INTRO {#the-intro-tag}

Un ficheiro pode levar o seu tempo de intro nunha etiqueta `INTRO`, en
segundos (`12.5`) ou como `m:ss`. O nome pode escribirse en calquera caso
(`INTRO`, `Intro`). Pode ser un cadro de texto de usuario ID3v2 (MP3, WAV,
AIFF, DSF), un comentario Vorbis, Opus ou FLAC, un elemento APE (WavPack,
Monkey's Audio) ou un átomo libre MP4. Léese durante a análise. Un fin de
intro manual segue gañando.

## Como se atopan os marcadores automáticos {#how-the-automatic-markers-are-found}

A análise mide os picos da pista en pasos de 10 ms e a súa sonoridade en
xanelas curtas (50 ms de forma predeterminada).

- **Cue in / cue out:** só se salta o case silencio do comezo e do final:
  todo o que teña un pico que alcance o *limiar de recorte* (−60 dBFS de
  forma predeterminada), en calquera canle, consérvase, cunha *marxe de
  recorte* (20 ms de forma predeterminada) arredor. Os fundidos de entrada
  suaves, as colas tranquilas e os sons curtos nunca se cortan.
- **MIX:** a análise atopa o último punto onde a pista aínda está menos da
  *caída para o segue* (15 dB de forma predeterminada) por debaixo da súa
  propia sonoridade típica, de modo que masters fortes e suaves co mesmo
  fundido mesturan igual. Ese punto nunca está a máis da *duración máxima da
  mestura* (4 s de forma predeterminada) antes do cue-out, así que as
  superposicións son curtas.
- **Outro:** a análise explora cara atrás desde o cue-out e atopa onde o nivel
  cae máis da *caída de nivel do outro* (6 dB de forma predeterminada) por
  debaixo da sonoridade mediana da pista. O outro nunca é máis longo de 30 s
  de forma predeterminada.
- As pistas máis curtas que a *duración mínima para os marcadores de mestura
  e outro* (60 s de forma predeterminada), como jingles e anuncios, non teñen
  MIX nin outro.

### Gravacións longas {#long-recordings}

Un programa enteiro (unha, catro ou máis horas) analízase como unha canción,
mentres se reproduce se é preciso: un ficheiro FLAC ou Opus de 4 horas leva
menos dun minuto nun computador actual, e a memoria non crece coa duración.
Buscar calquera punto, incluso preto do final, é instantáneo. A forma de onda
e os marcadores consérvanse na caché de análise ata unhas 16 horas de audio;
un ficheiro máis longo tamén funciona, pero analízase de novo cada vez que
inicia Fauste Player.

Todos estes valores están en **Configuración → Análise**. Despois de
cambialos, as pistas analízanse de novo automaticamente.

## Que fai o reprodutor con eles {#what-the-player-does-with-them}

- **Modo continuo con mestura automática activada:** no punto MIX a pista
  seguinte comeza a pleno nivel mentres a actual se esvaece ata o seu
  cue-out. A superposición é exacta ao nivel de mostra.
- **Modo continuo sen punto MIX,** ou con mestura automática desactivada: a
  pista seguinte comeza exactamente no cue-out, sen pausa.
- **Modo single**, ou **Stop ao final**: o reprodutor detense no cue-out.
- **Premer Play en antena:** a pista seguinte comeza ao instante e a actual
  esvaécese durante a *duración do fundido* (1 s de forma predeterminada).
- **Usar cue-in e cue-out desactivado** (Configuración → Reprodutores): cada
  reprodutor reproduce cada pista de 0 ata o final do ficheiro. O cue-in e o
  cue-out, automáticos e manuais, consérvanse, e a forma de onda debúxaos
  como liñas atenuadas. O punto MIX, a intro e o outro seguen funcionando,
  dentro do ficheiro enteiro; a **Mestura automática** é un interruptor á
  parte. As contas atrás, a columna de duración, os totais das listas en
  Configuración e os tempos da API remota seguen o mesmo rango. Os cartuchos
  usan sempre o seu propio cue-in e cue-out. Cambiar o axuste nunca reinicia,
  busca nin detén unha pista que se está reproducindo; a pista seguinte
  prepárase de novo.

Unha pista pódese reproducir antes de que remate a súa análise. Ata entón
reprodúcese desde o comezo ata o final do ficheiro, sen punto MIX.

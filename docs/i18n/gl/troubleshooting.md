# Resolución de problemas

## Sen son {#no-sound}

1. Abre **Configuración → Saídas de audio** e preme **Probar Main** para o
   reprodutor. Se escoitas o ton, comproba o fader de volume do reprodutor.
2. Se non escoitas nada, escolle outro dispositivo ou par de canles. Os
   cambios nas saídas teñen efecto despois dun reinicio: preme **Reiniciar
   agora** en Configuración.
3. En Linux, prefire **PipeWire** ou **PulseAudio** en Configuración → Saídas
   de audio → Sistema de audio. Comparten a tarxeta de son con outros
   programas. **ALSA** fala coa tarxeta directamente e pode atopala ocupada.
4. **JACK** aparece como non dispoñible («no output device») cando non hai
   ningún servidor JACK en execución. Inicia o servidor (por exemplo con
   QjackCtl) e reinicia a aplicación. Axusta o servidor JACK á frecuencia de
   mostraxe de Configuración (48 kHz de forma predeterminada): JACK funciona a
   unha soa frecuencia para todos os programas.
5. **PipeWire** non se ofrece nos arquivos descargables; chegan a PipeWire
   polo seu servizo PulseAudio, que funciona igual. Está dispoñible nas
   compilacións feitas coa funcionalidade `pipewire`.

## Bit-perfect {#bit-perfect}

- **O distintivo BP queda apagado.** Comproba cada condición en
  [Saída bit-perfect](bit-perfect.md#the-bp-badge): volume ao 100 %, sen
  fundido, nada máis nas mesmas saídas, un ficheiro sen perdas que fose
  analizado, e un dispositivo que funcione á frecuencia do ficheiro.
- **Un silencio curto antes dunha pista.** O dispositivo bit-perfect
  reabriuse á frecuencia de mostraxe da pista. Mantén a biblioteca a unha
  soa frecuencia para evitalo.
- **Unha pista reprodúcese remostreada, e o rexistro di que o dispositivo
  está ocupado (Linux).** Para cambiar de frecuencia, a aplicación pecha o
  dispositivo e ábreo de novo. Nese momento o servidor de son (PipeWire)
  pode quedarse coa tarxeta. A aplicación tenta de novo algunhas veces; se a
  tarxeta segue ocupada, a pista reprodúcese á frecuencia actual do
  dispositivo, e a pista seguinte pide a súa frecuencia outra vez. Para darlle
  a tarxeta só á aplicación, abre a configuración de son do sistema e pon o
  perfil desa tarxeta en **Off** (ou **Pro Audio**), para que o servidor de
  son deixe en paz o seu dispositivo `hw:`. O número de intentos e a espera
  entre eles son `tuning.device_busy_retries` e `tuning.device_busy_retry_ms`
  no ficheiro de configuración.
- **O dispositivo reproduce, pero o distintivo BP queda apagado (Windows ou
  macOS).** Rexeitouse o acceso exclusivo, e o dispositivo reproduce
  compartido.
  - Windows: pode que outro programa teña o dispositivo en exclusiva, ou que o
    control exclusivo estea desactivado nas propiedades Avanzadas do
    dispositivo.
  - macOS: pode que outro programa teña o dispositivo en modo hog, ou que o
    dispositivo ofreza as súas frecuencias só como un rango continuo (a
    maioría das interfaces listan frecuencias fixas).
- **Un dispositivo `hw:` non se pode abrir (Linux).**
  - Pode que un servidor de son teña a tarxeta. Páraa, ou configura o servidor
    para que deixe en paz esa tarxeta, e reinicia a aplicación.
  - Algúns DAC USB só aceptan mostras de 24 bits empaquetadas (`S24_3LE`), que
    a biblioteca de audio non admite. Usa esa tarxeta a través de `plughw:`
    (non bit-perfect) en cambio.

### DSD {#dsd}

- **Unha pista DSD reprodúcese convertida aínda que o dispositivo está en DoP
  ou DSD nativo.** O rexistro di por que («DSD converted to PCM» e a razón)
  nestes casos: o volume do reprodutor non está ao 100 %, soa algo máis no
  dispositivo, hai máis de dúas canles, ou o dispositivo rexeita a frecuencia
  (DoP necesita a frecuencia DSD dividida por 16, por exemplo 176,4 kHz para
  DSD64) ou non ten un formato de 24 ou 32 bits. Unha pista que aínda non foi
  analizada convértese en silencio, sen liña de rexistro: analízaa
  (Configuración → Análise) e reprodúcea de novo.
- **Só a primeira pista dun álbum DSD sae como DSD.** Ese é o axuste de
  mestura predeterminado: as pistas que o reprodutor inicia por si só
  reprodúcense convertidas. Escolle **Manter o DSD e silenciar as outras
  fontes** en Configuración → Saídas de audio para mantelas en DSD. Consulta
  [DSD](bit-perfect.md#dsd).
- **A cabeceira mostra DSD pero o conversor reproduce ruído ou non se
  engancha.** O conversor non recoñece DoP (nin o formato nativo). Volve
  poñer o dispositivo en **Converter a PCM**.
- **Un chasquido cando unha pista DSD empeza, para ou deixa o DSD.** O
  conversor precisa máis silencio DSD: sube **Silencio DSD** (200 ms de forma
  predeterminada) en Configuración → Saídas de audio, Avanzado.
- **Outros reprodutores ou cartuchos están en silencio no dispositivo.** Unha
  pista DSD está a soar con **Manter o DSD e silenciar as outras fontes**;
  móstrase o distintivo **Outras silenciadas**. Volven soar cando remata a
  pista.

## Aviso «Saída perdida» {#output-lost-alert}

A barra de estado mostra **Saída perdida: &lt;dispositivo&gt;** cando un
dispositivo deixa de responder. Os reprodutores seguen contando e mesturando
cun reloxo interno, así que a automatización non se detén. O dispositivo
tenta de novo cada 2 segundos e volve tomar o relevo cando regresa. Volve conectar o
cable ou acende de novo a interface.

### «Saída perdida» que nunca desaparece, cunha saída `hw:` directa {#output-lost-that-never-clears-with-a-direct-hw-output}

Unha tarxeta de son usada a través dunha saída ALSA `hw:` directa (por
exemplo unha saída bit-perfect) está en poder só de Fauste Player: o
servidor de son (PipeWire ou PulseAudio) non a pode usar á vez. Se outra
saída pasa polo dispositivo predeterminado do servidor de son e ese
dispositivo predeterminado é a mesma tarxeta, esa saída nunca arranca e
queda en **Saída perdida**. O rexistro di unha vez «output device opened but
never started».

Usa un só camiño por tarxeta: dirixe todas as saídas desa tarxeta polo mesmo
dispositivo `hw:` (con canles distintas se fai falta), ou escolle outra
tarxeta como saída predeterminada do servidor de son na configuración de son
do teu sistema.

## Unha pista mostra unha icona de aviso ou un ficheiro cunha cruz {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

O ficheiro falta (movido, eliminado, desmontado: un ficheiro cunha cruz) ou
non se pode decodificar (un sinal de aviso). Os reprodutores sáltano. Pon o
punteiro sobre a fila para ver cal é, e a ruta do ficheiro.

Un ficheiro que falta búscase de novo cada 30 segundos
(`tuning.missing_recheck_ms` no ficheiro de configuración): cando se monta a
unidade ou se volve poñer o ficheiro, a pista pasa a ser reproducible por si
soa. Un ficheiro que non se pode decodificar comproba de novo por si só co
mesmo temporizador, por tamaño e data de modificación: non se decodifica de
novo salvo que un deles cambiase, por exemplo cando remata unha copia. Para
comprobalo ao instante usa **Volver analizar** no menú da súa fila, ou
**Configuración → Análise → Volver analizar todas as pistas** para toda a
biblioteca.

## Cortes de audio {#audio-dropouts}

A barra de estado avisa durante 5 segundos despois de cada corte que detecta a
aplicación: **P1: cortes de audio (3)** cando a decodificación dun reprodutor
non puido seguir o ritmo do disco (a conta é da pista que soa agora), e
**&lt;dispositivo&gt;: cortes do dispositivo de audio (2)** cando o
dispositivo de saída non chegou a tempo (un xrun). O rexistro tamén anota cada
un, como máximo unha liña cada 10 segundos por tipo, con cantos houbo. Non
todos os sistemas de audio informan de xruns (PulseAudio non o fai; o modo
exclusivo de Windows tampouco).


- Aumenta o **tamaño do búfer** en Configuración (e preme **Reiniciar
  agora**).
- En Linux, permite a planificación en tempo real. A aplicación pídella ao
  sistema a través de rtkit (D-Bus). Tamén funciona pertencer ao grupo `audio`
  cun límite `rtprio`.
- Evita as unidades de rede para a música que se reproduce en antena.

## «A interface tivo un erro» {#the-interface-hit-an-error}

Detectouse un erro de debuxo. O audio non se ve afectado. Preme **Reiniciar a
interface**. Infórmanos del co rexistro.

## Rexistros e informes de fallos {#logs-and-crash-reports}

Consulta [Datos e copias de seguranza](data-and-backups.md) para ver o
cartafol de rexistros. Hai un ficheiro de rexistro por día, e consérvanse os
últimos 14. Os informes de fallos gárdanse como `crash-<time>.txt`. Define
`RUST_LOG=debug` na contorna para ter máis detalle. Adxunta os dous ficheiros
ao informar dun erro.

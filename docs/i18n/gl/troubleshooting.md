# Resolución de problemas

## Sen son {#no-sound}

1. Abre **Configuración → Saídas de audio** e preme **Probar Main** para o
   reprodutor. Se escoitas o ton, comproba o fader de volume do reprodutor.
2. Se non escoitas nada, escolle outro dispositivo ou par de canles. Un
   cambio nas saídas aplícase cando nada soa nelas; **Aplicar agora** en
   Configuración aplícao ao instante.
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
  [Saída bit-perfect](bit-perfect.md#the-bp-badge): sen fundido, nada máis
  nas mesmas saídas, un ficheiro sen perdas que fose analizado, e un
  dispositivo que funcione á frecuencia do ficheiro.
- **Un silencio curto antes dunha pista.** O dispositivo bit-perfect
  reabriuse á frecuencia de mostraxe da pista. Só pasa cando cambia a
  frecuencia: as pistas á mesma frecuencia séguense sen oco. Mantén a
  biblioteca a unha soa frecuencia para evitalo. Consulta
  [O que custa](bit-perfect.md#what-it-costs).
- **As pistas non se mesturan, e Play non fai fundido.** Nun reprodutor cuxa
  saída Main é bit-perfect, cada pista reprodúcese ata o seu final e despois
  comeza a seguinte, e Play / Seguinte e Anterior cambian ao instante: nese
  reprodutor non hai fundidos cruzados nin segues. Desactiva bit-perfect para
  o dispositivo, ou move o reprodutor a outra saída, para volver mesturar.
  Consulta [O que custa](bit-perfect.md#what-it-costs).
- **O fader de volume non se move.** A saída Main do reprodutor é
  bit-perfect, o que mantén o volume ao 100 %; a dica do fader dío. Os faders
  MIDI e a API remota tampouco o poden mover. Consulta
  [O que custa](bit-perfect.md#what-it-costs).
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
cun reloxo interno, así que a automatización non se detén. Vólvese tentar o
dispositivo cada 2 segundos e retoma o relevo cando regresa. Volve conectar o
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

Un WAV ou FLAC que aparece como ilexible, ou que soaría como ruído forte, pode
ser unha extracción de CD DTS: o ficheiro contén un fluxo de bits DTS en lugar
de audio. A aplicación rexéitao (o rexistro di «encoded bitstream stored as
PCM») e non decodifica DTS. Convértoo primeiro a PCM normal con outra
ferramenta.

## Un cartafol vixiado mostra un aviso {#a-watched-folder-shows-a-warning}

Unha categoría que vixía un cartafol mostra ⟳, doutra cor cando o cartafol non
se segue; pon o punteiro enriba para ver a razón. Consulta
[Cartafoles vixiados](library.md#watched-folders).

- **Non se atopa ou non se pode ler**: a unidade ou o recurso compartido non
  está montado ou non se alcanza. Un cartafol baleiro que debería conter pistas
  da biblioteca conta tamén como non atopado, porque un recurso compartido sen
  montar adoita parecer un. Non se marca nin se quita nada; o cartafol
  búscase de novo cada 30 segundos e examínase cando aparece. Se baleiraches o
  cartafol adrede, quita as súas pistas seleccionándoas na táboa da biblioteca
  e escollendo **Quitar da biblioteca…**, ou escollendo **Volver analizar**
  nelas, o que marca como ausentes as que perderon o ficheiro para que
  **Quitar pistas que faltan…** poida quitalas.
- **Non se seguen os cambios**: o sistema non quixo vixialo, normalmente un
  cartafol con moitísimos subcartafoles. O cartafol examínase igualmente en
  cada inicio e en cada reintento, así que os ficheiros novos aparecen
  entón, non ao instante. En Linux, unha árbore de cartafoles máis grande ca o
  límite de vixilancias por usuario precisa un
  `fs.inotify.max_user_watches` máis alto (por exemplo `sudo sysctl
  fs.inotify.max_user_watches=524288`; faino permanente en
  `/etc/sysctl.d/`).
- **Non é unha ruta completa**: o cartafol gardado editouse a man cara a unha
  ruta que non está completa. Escolle **Cambiar cartafol vixiado…** para
  escollelo de novo.
- **Un recurso compartido de rede non mostra aviso, pero os ficheiros engadidos
  desde outro computador non aparecen**: nun recurso de rede (SMB/CIFS, NFS)
  o sistema adoita aceptar a vixilancia pero só informa dos cambios feitos
  desde este computador. Os cambios feitos noutro lugar vense no seguinte
  inicio, non mentres a aplicación funciona.
- **Un cartafol con miles de ficheiros** tarda un pouco en examinarse ao
  iniciar; o progreso da análise móstrao.
- As pistas cuxo ficheiro saíu dun cartafol vixiado quedan na biblioteca
  marcadas como ausentes. Quítaas con **Quitar pistas que faltan…** no menú da
  categoría.

## Cortes de audio {#audio-dropouts}

A barra de estado avisa durante 5 segundos despois de cada corte que detecta a
aplicación: **P1: cortes de audio (3)** cando a decodificación dun reprodutor
non puido seguir o ritmo do disco (a conta é da pista que soa agora), e
**&lt;dispositivo&gt;: cortes do dispositivo de audio (2)** cando o
dispositivo de saída non chegou a tempo (un xrun). O rexistro tamén anota cada
un, como máximo unha liña cada 10 segundos por tipo, con cantos houbo. Non
todos os sistemas de audio informan de xruns (PulseAudio non o fai; o modo
exclusivo de Windows tampouco).


- Aumenta o **tamaño do búfer** en Configuración (aplícase cando nada soa
  nese dispositivo, ou ao instante con **Aplicar agora**).
- A través dun servidor de son (PulseAudio, o dispositivo ALSA `default`),
  cada saída xa pide polo menos 50 ms sexa cal for o tamaño do búfer, para
  que un computador ocupado non se note; Configuración móstrao ao carón da
  latencia. Nun computador vello ou moi cargado, sube `shared_buffer_ms` na
  sección `outputs` do ficheiro de configuración (por exemplo a 100). O
  sistema mantén en cola uns dous deses bloques, así que os botóns dos
  cartuchos e de play soan entón uns dous bloques máis tarde. Para a saída en
  antena, a entrada de hardware propia do dispositivo (`hw:` en Linux) evita
  por completo o servidor de son.
- En Linux, permite a planificación en tempo real. A aplicación pídella ao
  sistema a través de rtkit (D-Bus), e o rexistro anota un rexeitamento.
  Moitos escritorios rexéitanllo ás aplicacións.
- Evita as unidades de rede para a música que se reproduce en antena.

## A interface vai lenta nun computador vello {#the-interface-is-slow-on-an-old-computer}

Mentres algo soa, a interface debuxa como máximo 30 fotogramas por segundo
(`max_fps` na sección `ui` do ficheiro de configuración), e os clics e as
teclas respóndense ao instante. Nun computador vello ou ocupado, baixa ese
valor (por exemplo a 20) para deixar máis procesador ao audio; 0 debuxa todos
os fotogramas que mostra a pantalla. Os vúmetros miden o mesmo a calquera
ritmo; só se debuxan con menos frecuencia.

## Non se poden soltar ficheiros en Linux {#files-cannot-be-dropped-on-linux}

A xanela só acepta ficheiros arrastrados desde un xestor de ficheiros cando
funciona en X11. De forma predeterminada faino, tamén nun escritorio Wayland
(a través de XWayland, que ten todos os escritorios). Se `ui.window_system` en
`config.json` está en `"Wayland"`, a xanela é unha xanela Wayland nativa, que
pode verse máis nítida con escalado de pantalla fraccionario, pero non se
poden soltar ficheiros nela: usa **+ Engadir** no seu lugar. Volve poñelo en
`"Auto"` (o predeterminado) ou `"X11"` e reinicia Fauste Player.

## Un controlador MIDI non responde {#a-midi-controller-does-not-respond}

Se **Configuración → MIDI** mostra o controlador como conectado pero os seus
botóns e faders non fan nada, preme alí **Reconectar portos**: o programa
pecha os portos MIDI e ábreos de novo, como faría desenchufar e volver
enchufar o controlador. Se iso non axuda, desenchufa o controlador e volve
enchufalo.

En ambos os casos, o rexistro di o que pasou. Lista os portos MIDI cada vez
que cambian, cada porto aberto e pechado (con por que), un porto que non se
pode abrir (unha vez, ata que se abre ou desaparece), a primeira mensaxe
recibida en cada conexión, a primeira mensaxe ignorada en cada conexión coa
razón (sen asignación, un fader que aínda non alcanzou o volume, unha acción
que agora non pode actuar) e a primeira mensaxe de LED que fallou. Un
controlador conectado que nunca rexistra unha primeira mensaxe non envía nada
que o programa reciba. Para ver cada mensaxe ignorada e cada reintento
fallido, inicia o programa con `RUST_LOG=info,fp_control=debug`.

## «A interface tivo un erro» {#the-interface-hit-an-error}

Detectouse un erro de debuxo. O audio non se ve afectado. Preme **Reiniciar a
interface**. Infórmanos del co rexistro.

## Rexistros e informes de fallos {#logs-and-crash-reports}

Consulta [Datos e copias de seguranza](data-and-backups.md) para ver o
cartafol de rexistros. Hai un ficheiro de rexistro por día. As liñas idénticas
que se repiten unha tras outra escríbense unha soa vez, seguidas dunha liña
como `previous message repeated 12 times` (durante unha serie longa de
repeticións esa liña escríbese cada minuto). As liñas que difiren, por
exemplo porque levan un contador, non se agrupan. As mensaxes rutineiras de
conexión do cliente PulseAudio omítense de forma predeterminada. Define
`RUST_LOG=debug` na contorna para ter máis detalle (substitúe o filtro
predeterminado, `info,pulseaudio=warn`).

O rexistro ten límite: o ficheiro dun día deixa de crecer aos 10 MiB (a súa
última liña dío; o resto dese día non se rexistra, que é o que provoca unha
avalancha de liñas diferentes) e consérvanse os últimos 14 ficheiros.
Cámbianse no ficheiro de configuración con `limits.max_log_file_mb` (1 a 1024),
`limits.max_log_files` (2 a 365) e `limits.log_repeat_summary_secs` (1 a 3600,
60 de forma predeterminada, con que frecuencia se informa dunha serie longa de
repeticións); aplícanse no seguinte inicio. Os informes de fallos gárdanse
como `crash-<time>.txt` no mesmo cartafol e teñen un límite á parte
(`limits.max_crash_reports` por execución).
Adxunta os dous ficheiros ao informar dun erro.

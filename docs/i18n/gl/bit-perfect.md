# Saída bit-perfect

Un dispositivo **bit-perfect** recibe as mostras de cada ficheiro exactamente
como están no ficheiro: a mesma frecuencia de mostraxe, os mesmos valores, sen
remostraxe, cambio de volume nin mestura. É útil en cadeas de monitorización
e ligazóns dixitais, onde se debe evitar calquera procesamento no computador.

## Configurar un dispositivo como bit-perfect {#setting-a-device-bit-perfect}

1. En **Configuración → Saídas de audio**, escolle o dispositivo de forma
   explícita para a saída Main dun reprodutor (ou a da cartucheira). Un
   reprodutor deixado no predeterminado do sistema non pode facerse
   bit-perfect. Un dispositivo que xa non usa ningunha saída perde o seu
   interruptor bit-perfect e o seu modo DSD no seguinte inicio da
   aplicación.
2. Escolle **Avanzado** na parte de arriba da sección. En **Axustes por
   dispositivo**, activa **Bit perfect** ao carón do dispositivo.
3. Reinicia a aplicación.

O dispositivo empeza entón á súa propia frecuencia de mostraxe se lle deches
unha no mesmo lugar, e se non á frecuencia de mostraxe global, e desde aí
segue cada ficheiro.

O interruptor está desactivado cando o dispositivo non pode dar acceso
exclusivo.
- **Linux:** escolle un dispositivo ALSA cuxo nome comece por `hw:`. É a
  propia tarxeta de son. PulseAudio, PipeWire, JACK e os dispositivos ALSA
  `default` ou `plughw:` mesturan ou converten, así que nunca son
  bit-perfect.
- **Windows:** escolle o dispositivo no sistema **WASAPI**. Ábrese en modo
  exclusivo.
  - Na configuración de son de Windows, as propiedades **Avanzadas** do
    dispositivo deben ter activada a opción *Permitir que as aplicacións
    tomen o control exclusivo deste dispositivo* (está activada de forma
    predeterminada).
  - Mentres reproduce, ningún outro programa pode usar o dispositivo.
- **macOS:** escolle o dispositivo en **Core Audio**. Ábrese en modo hog.
  - A frecuencia de mostraxe do dispositivo axústase á da pista, e o seu
    formato ao formato enteiro máis ancho que ofrece a esa frecuencia (os
    axustes que mostra Configuración de Audio e MIDI).
  - Devólvense cando a aplicación deixa de usar o dispositivo.
  - Dous dispositivos exactamente co mesmo nome non se poden facer
    bit-perfect.

## Que pasa nun dispositivo bit-perfect {#what-happens-on-a-bit-perfect-device}

- **Acceso exclusivo.** Nada máis no computador pode reproducir no dispositivo
  mentres a aplicación o usa. Se se rexeita o acceso exclusivo, o dispositivo
  reproduce igualmente, compartido, e o distintivo BP queda apagado.
- **A frecuencia segue o ficheiro.** Cando non se reproduce nada no
  dispositivo e empeza unha pista a outra frecuencia de mostraxe, o
  dispositivo ábrese de novo a esa frecuencia.
  - Isto sucede cando reproduces unha pista, retomas unha cargada en pausa,
    preescoitas ou disparas un cartucho. As pistas que só están á espera (a
    pista seguinte de cada reprodutor) prepáranse de novo á nova frecuencia.
  - A reapertura leva o que o dispositivo precise para arrancar (normalmente
    unhas poucas decenas de milisegundos). O comezo retrásase outro tanto.
  - Mentres algo se reproduce no dispositivo, a frecuencia nunca cambia. Unha
    pista a outra frecuencia que empeza entón (por exemplo unha pista de
    48 kHz á que se mestura despois dunha de 44,1 kHz, ou unha pista iniciada
    mentres outro reprodutor ou un cartucho soa no mesmo dispositivo)
    convértese durante toda a súa duración, e non é bit-perfect.
  - Se o dispositivo rexeita unha frecuencia, conserva a anterior e a pista
    convértese.
- **Sen procesamento, cando nada o pide.** As mostras pasan sen cambios mentres
  se cumpran todas estas condicións:
  - o volume do reprodutor está ao 100 %;
  - non hai ningún fundido en curso;
  - non soa nada máis nas mesmas saídas (outro reprodutor, un cartucho, un
    ton de proba).

## DSD {#dsd}

Un ficheiro DSD normalmente reprodúcese convertido a PCM, como calquera outro
ficheiro. Un dispositivo bit-perfect pode recibir en cambio o fluxo DSD sen
cambios.

**Os tres modos.** Na vista Avanzado de Configuración → Saídas de audio, cada
dispositivo que usa unha saída ten unha opción **DSD** baixo o seu
interruptor bit-perfect:
- **Converter a PCM** (o predeterminado): o DSD convértese, como en calquera
  outro dispositivo.
- **DoP** (DSD over PCM): os bits DSD viaxan dentro de mostras PCM de 24 bits,
  que a maioría dos conversores con DSD recoñecen. Funciona en todos os
  sistemas.
- **DSD nativo** (só Linux): DSD en bruto, para dispositivos ALSA `hw:` cuxo
  controlador indica un formato de mostra DSD.

Só se ofrecen os modos que o dispositivo pode aceptar, e unha liña baixo a
opción di por que os outros non: o dispositivo non está conectado, bit-perfect
está desactivado, o dispositivo non se pode abrir en exclusiva, o DSD nativo
necesita Linux, ou o dispositivo non acepta DSD nativo. Un modo gardado para
un dispositivo que agora non o pode aceptar aparece como PCM, que é o que se
reproduce; o modo gardado volve cando o dispositivo o pode aceptar de novo.
Cambiar un modo, o axuste de mestura ou o silencio DSD necesita un reinicio,
como os outros axustes de saída.

**Cando o DSD sae sen cambios.** Todo isto debe cumprirse cando empeza a
pista:
- o dispositivo é bit-perfect, con acceso exclusivo, e o seu modo é DoP ou DSD
  nativo;
- a pista é DSD (DSF ou DFF), mono ou estéreo, e foi analizada (así se coñece
  a súa frecuencia DSD);
- o volume do reprodutor está ao 100 %;
- non soa nada máis no dispositivo (outro reprodutor, un cartucho, un ton de
  proba);
- o dispositivo acepta o fluxo. DoP necesita unha frecuencia de dispositivo
  igual á frecuencia DSD dividida por 16 (176,4 kHz para DSD64, 352,8 kHz
  para DSD128, 705,6 kHz para DSD256) e un formato de 24 ou 32 bits. O DSD
  nativo necesita un dispositivo que acepte o formato DSD a esa frecuencia.

Cando remata o DSD nativo, o dispositivo volve a PCM á frecuencia que tiña
antes da pista DSD, porque moitos conversores aceptan DSD nativo a
frecuencias que non poden reproducir como PCM (ningún conversor reproduce PCM
á frecuencia á que funciona DSD512). Unha pista que continúa como PCM conserva
a frecuencia do fluxo DSD cando o dispositivo a acepta como PCM, e se non
continúa á frecuencia anterior desde onde estaba, como todo o que se reproduce
nese dispositivo; un fundido en curso alí remata ao instante. O dispositivo
nunca queda nunha frecuencia que rexeita: se non se abre ningunha frecuencia
(por exemplo, o dispositivo foi desconectado nese momento), a saída perdese
ata que o reintento automático a abre de novo á frecuencia anterior.

Se non, a pista convértese a PCM e o rexistro di por que (por exemplo
«something else plays on the device» ou «the device refused 705600 Hz»). A
preescoita e os cartuchos sempre se converten.

Mentres o DSD sae sen cambios:
- o distintivo da cabeceira le **DSD** en lugar de **BP**;
- os vúmetros mostran o nivel da conversión a PCM da mesma pista, así que
  funcionan como de costume;
- o volume debe quedar ao 100 %: a dica do fader dío. Movelo cambia a pista a
  PCM (ver abaixo);
- Stop e un stop con fundido paran a pista ao instante, sen fundido, porque un
  fluxo DSD non se pode esvaecer. Premer Play noutra pista mentres soa córtaa
  do mesmo xeito en vez de facer un fundido cruzado;
- a pausa e a retomada tamén actúan ao instante, sen rampla.

**Silencio nos bordos.** Cada inicio, final e cambio a PCM envía primeiro
silencio DSD (200 ms de forma predeterminada), para que o conversor se
enganche sen chasquido. A excepción é unha pista DSD que continúa un fluxo do
mesmo tipo e frecuencia DSD cuxo silencio aínda está en curso: o conversor
aínda está enganchado, así que empeza sen silencio extra. Por iso unha pista
empeza ese tempo máis tarde, e un cambio a PCM deixa un oco desa duración. É
**Silencio DSD** en Configuración → Saídas de audio, Avanzado (de 0 a
2000 ms).

**Cando outra fonte necesita o dispositivo.** **Cando outra fonte necesita
unha saída DSD** en Configuración → Saídas de audio escolle o que pasa cando
outro reprodutor, un cartucho ou un ton de proba empeza no mesmo dispositivo
(mover o fader do propio reprodutor é a excepción: sempre cambia a pista a
PCM):
- **Continuar a pista DSD en PCM** (o predeterminado). O fluxo cambia a PCM
  despois do silencio DSD, e a pista continúa, convertida, desde onde estaba.
  O mesmo lle pasa á pista que segue por si soa (ver abaixo).
- **Manter o DSD e silenciar as outras fontes.** Nada interrompe o fluxo DSD.
  As outras fontes dirixidas ao dispositivo quedan silenciadas ata que remate
  a pista DSD, e mentres tanto o reprodutor mostra un distintivo **Outras
  silenciadas**. A pista seguinte do propio reprodutor non se superpón: empeza
  cando remata a pista DSD, sen fundido cruzado nin segue. Unha pista PCM
  agarda ao silencio DSD; unha pista DSD do mesmo tipo e frecuencia DSD
  continúa o fluxo sen el. Mover o fader segue cambiando a pista a PCM.

**Un álbum non segue en DSD co axuste predeterminado.** Con *Continuar a pista
DSD en PCM*, só sae como DSD unha pista DSD que empeza nun dispositivo
libre. As pistas que o reprodutor inicia por si só despois (ao final dunha
pista, un segue ou un fundido cruzado) empezan desde unha precarga, que é
sempre PCM, así que o dispositivo cambia a PCM e reprodúcense convertidas. Unha
pista que inicias ti (Play, dobre clic) sae como DSD de novo cando o
dispositivo está libre ou o fluxo DSD anterior aínda está no seu silencio coa
mesma frecuencia DSD. Para manter un álbum DSD enteiro como DSD, escolle
*Manter o DSD e silenciar as outras fontes*. Entón cada pista do reprodutor
sae como DSD, e a seguinte empeza cando remata a anterior.

Se o dispositivo se perde mentres soa DSD e volve sen poder levalo (por
exemplo sen acceso exclusivo), a pista continúa como PCM.

## O distintivo BP {#the-bp-badge}

O distintivo **BP** da cabeceira do reprodutor acéndese mentres a pista actual
chega sen cambios ao seu dispositivo Main. Todo isto debe cumprirse:

- o dispositivo é bit-perfect e está aberto con acceso exclusivo;
- o dispositivo funciona á frecuencia de mostraxe da pista;
- a pista é PCM enteiro sen perdas (WAV, AIFF, FLAC, ALAC, WavPack ou
  Monkey's Audio), mono ou estéreo, e de 24 bits como máximo, e o formato do
  dispositivo comporta o seu tamaño de mostra (un ficheiro de 24 bits nun
  dispositivo de 16 bits non é bit-perfect). O DSD convértese, así que nunca
  acende BP; cando sae sen cambios (consulta [DSD](#dsd)) o distintivo le
  **DSD** en cambio;
- a pista foi analizada, porque así se coñecen a súa frecuencia e o seu
  tamaño de mostra. As pistas que analizou unha versión anterior reciben o
  seu formato cando se analizan de novo (o aviso de inicio, ou Configuración →
  Análise), ou en canto un reprodutor as mostra ou un cartucho as ten;
- o volume está ao 100 %, non hai ningún fundido en curso, e non soa nada máis
  nas mesmas saídas.

Algúns ficheiros nunca se mostran como bit-perfect:
- **Ficheiros con perdas** (MP3, AAC, Ogg Vorbis, Opus): as súas mostras
  decodificadas non son os valores enteiros que acepta un dispositivo.
- **Ficheiros de máis de 24 bits:** o mesturador traballa en coma flotante de
  32 bits, que representa 24 bits con exactitude.
- **Ficheiros con máis de dúas canles:** mestúranse a estéreo.

## Comprobalo ti mesmo {#checking-it-yourself}

Para verificar unha cadea de punta a punta:

1. Conecta a saída dixital do dispositivo (S/PDIF, AES ou loopback USB) a un
   gravador que capture con exactitude de bit.
2. Reproduce un ficheiro de proba sen perdas ao 100 % sen que soe nada máis.
3. Grávao.
4. Compara a gravación co ficheiro. Por exemplo, con SoX, inverte un e
   mestúraos: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav` despois de
   aliñar os seus comezos. Cada mostra da diferenza debe ser cero.

As probas automáticas do proxecto comproban a mesma propiedade dentro da
aplicación, nun dispositivo simulado.

### DSD nun conversor real {#dsd-on-a-real-converter}

As probas automáticas só comproban o DSD en dispositivos simulados. O proxecto
non probou DoP nin DSD nativo nun conversor real. Para comprobar un:
1. Pon o dispositivo en **DoP** (ou **DSD nativo** en Linux), reinicia e
   reproduce un ficheiro DSD ao 100 % sen que soe nada máis. A cabeceira debe
   mostrar **DSD**, e a pantalla do propio conversor debería mostrar a
   frecuencia DSD (por exemplo DSD64) en vez dunha frecuencia PCM. Un
   conversor que mostra unha frecuencia PCM ou reproduce ruído non recoñece o
   fluxo: volve a **Converter a PCM**.
2. Escoita se hai un chasquido ou unha ráfaga de ruído ao comezo, no Stop, ao
   final da pista e ao mover o fader. Un chasquido significa que o conversor
   precisa un **Silencio DSD** máis longo (Configuración → Saídas de audio,
   Avanzado).
3. Inicia un cartucho ou outro reprodutor no mesmo dispositivo, unha vez con
   cada axuste de mestura, e comproba o comportamento descrito arriba.
4. En Linux, para comprobar o DSD nativo sen a aplicación, executa
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Abre o dispositivo en DSD nativo a DSD64 e reproduce un segundo de silencio
   DSD. Debe pasar, e o conversor debería enganchar a DSD64.
5. Con ficheiros DSD en `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   reprodúceos a través do motor nun dispositivo simulado e compara as
   palabras cos bytes do ficheiro (consulta [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).

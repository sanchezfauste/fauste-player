# Superfícies de control MIDI

Fauste Player es pot tocar des de controladors MIDI: controladors de pads i
faders, superfícies d'estil DJ o teclats. Els botons de transport i el fader
de volum de cada reproductor es poden assignar a un botó, una tecla o un
fader, i els botons amb llums mostren què fa cada reproductor.

## Activar-lo {#turning-it-on}

![Configuració, MIDI: l'interruptor per activar el MIDI, i la llista d'accions amb un botó Aprendre cadascuna](../../images/guide/settings-midi.png)

Obre **Configuració → MIDI** i marca **Usar superfícies de control MIDI**. La
llista sota **Ports d'entrada** mostra totes les entrades MIDI que té
l'ordinador i si estan connectades. Només s'obren els controladors que has
assignat (alguns sistemes donen un port a un sol programa alhora), més totes
les entrades mentre aprens un control; el programa mai no escolta els seus
propis ports. Els controladors es poden connectar i desconnectar mentre el
programa s'executa: cada un parell de segons es tornen a cercar els ports, i
un controlador que torna es connecta pel seu nom, amb les llums tornades a
ajustar.

A Linux, el MIDI passa per ALSA: el teu usuari ha de poder obrir el
seqüenciador (`/dev/snd/seq`, normalment per ser al grup `audio`).

## Assignar un control {#binding-a-control}

Per a cada reproductor hi ha una fila per acció: **Play / Següent**,
**Pausa**, **Stop**, **Stop amb fos**, **Reiniciar**, **Anterior**, **CUE** i
**Volum**.

1. Fes clic a **Aprendre** a la fila.
2. Prem el botó o mou el fader que vulguis (mentrestant diu **Mou un
   control…**). Un botó accepta una tecla, un pad o un botó que envia un canvi
   de control; **Volum** accepta un fader o un potenciòmetre (un canvi de
   control) o un fader de pitch bend.
3. La fila mostra el dispositiu i el control, per exemple
   `APC mini · Nota 36, canal 1`.

Si aquell control ja estava assignat a una altra acció, passa a aquesta. `Esc`
o fer clic de nou al botó cancel·la l'aprenentatge. **Esborrar** treu una
assignació.

## Com es comporten els controls {#how-the-controls-behave}

- Un botó actua quan es prem (una tecla o un pad que baixa, o un canvi de
  control que puja pel mig del seu interval), exactament com el botó del
  reproductor a la pantalla. Un botó atenuat a la pantalla no fa res.
- Un fader mou el volum a la mateixa escala que el fader de la pantalla. Per
  evitar salts, només n'assumeix el control quan arriba al volum actual o el
  passa (soft takeover); si el volum es canvia a la pantalla, el fader l'ha de
  tornar a atrapar. No canvia res fins que mous un control: iniciar el
  programa mai no posa res en antena.
- Amb **Encendre els botons (LED)** activat, els botons assignats
  s'il·luminen: Play mentre el reproductor és en antena, Pausa parpellejant
  mentre és en pausa, CUE mentre preescoltes, i Stop, Stop amb fos,
  Reiniciar i Anterior mentre poden actuar. Les llums van al port de sortida
  del controlador amb el mateix nom; se'n pot definir un de diferent a
  `config.json` (`midi.devices`).

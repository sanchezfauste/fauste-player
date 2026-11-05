# Primeiros pasos

## Instalación {#install}

En Windows, Fauste Player está na **Microsoft Store**
(<https://apps.microsoft.com/detail/9NH4KCL73463>), e en Linux na
**Snap Store** (<https://snapcraft.io/fauste-player>). Ou descarga un paquete
para o teu sistema da páxina **Releases** do proxecto.
Cada ficheiro ten un `.sha256` ao carón. Para comprobar unha descarga,
executa `sha256sum -c <file>.sha256` en Linux, ou
`shasum -a 256 -c <file>.sha256` en macOS.

### Linux {#linux}

| Paquete | Instalar | Actualizar | Desinstalar |
|---|---|---|---|
| Snap (Snap Store) | `sudo snap install fauste-player` | automática (`sudo snap refresh fauste-player` para actualizar agora) | `sudo snap remove fauste-player` |
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instala o `.deb` máis novo do mesmo xeito | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instala o `.rpm` máis novo | `sudo dnf remove fauste-player` |
| AppImage (calquera distribución) | `chmod +x fauste-player-<version>-x86_64.AppImage` e despois execútao | substitúe o ficheiro | elimina o ficheiro |
| Paquete Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instala o paquete máis novo | `flatpak uninstall com.sanchezfauste.FaustePlayer` |
| Arquivo | extráeo e executa `fauste-player` | extrae o máis novo | elimina o cartafol |

- **Integración co escritorio:** os paquetes engaden **Fauste Player** ao
  menú de aplicacións e permiten abrir con el listas `.m3u`, `.m3u8` e `.pls`,
  que se importan como listas novas.
- **Bibliotecas necesarias:** as bibliotecas ALSA e D-Bus. Todos os
  escritorios as teñen, e os paquetes decláranas. JACK úsase cando está
  instalado, e nunca é obrigatorio.
- **AppImage:** se non arranca porque falta FUSE, execútao con
  `--appimage-extract-and-run`.
- **Snap:** o snap funciona nun sandbox. De serie le o teu cartafol persoal
  (salvo os ficheiros e cartafoles ocultos), reproduce a través de PulseAudio
  ou PipeWire, e pode usar a rede para a API de control remoto e OSC (ambos
  desactivados ata que os actives). Dúas funcións precisan un permiso que
  concedes unha vez, desde un terminal:
  - Saída bit-perfect en dispositivos ALSA `hw:`, e superficies de control
    MIDI: `sudo snap connect fauste-player:alsa`.
  - Música en unidades USB e outros discos (`/media`, `/run/media`, `/mnt`):
    `sudo snap connect fauste-player:removable-media`.

  `snap connections fauste-player` mostra o que está conectado. Reinicia
  Fauste Player despois de conectar. A música noutros lugares (fóra do teu
  cartafol persoal e eses tres) non a pode ler o snap: usa outro paquete para
  ela. JACK non está dispoñible no snap. A súa configuración e as súas listas
  gárdanse en `~/snap/fauste-player/` (consulta [Datos e copias de
  seguranza](data-and-backups.md)).
- **Flatpak:**
  - O paquete necesita o runtime de freedesktop de Flathub. Se o remoto de
    Flathub non está configurado, executa primeiro
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - O sandbox pode ler o teu cartafol persoal (para reproducir a túa música
    onde está).
  - Reproduce a través de PulseAudio, ou directamente por ALSA para os
    dispositivos bit-perfect.
  - Pode usar a rede, para a API de control remoto e OSC (ambos
    desactivados ata que os actives).
  - Os paquetes ata a 0.10 instalábanse como `org.fauste.FaustePlayer`, e os
    máis novos instálanse como `com.sanchezfauste.FaustePlayer`, unha
    aplicación distinta. Para conservar a túa configuración, listas e
    cartuchos, pecha a aplicación e executa
    `mv ~/.var/app/org.fauste.FaustePlayer ~/.var/app/com.sanchezfauste.FaustePlayer`
    antes de iniciar a nova, e despois `flatpak uninstall org.fauste.FaustePlayer`.
- **Bibliotecas do escritorio:** a xanela usa libxkbcommon e EGL ou OpenGL,
  con Wayland ou X11. Todos os escritorios as teñen, e o `.deb` e o `.rpm`
  decláranas. Nun sistema moi mínimo, instálaas antes de usar o AppImage.

### Windows {#windows}

O xeito máis doado é a **Microsoft Store**: busca *Fauste Player*, ou abre
<https://apps.microsoft.com/detail/9NH4KCL73463>, e escolle **Obter**. A
Store mantena actualizada, e **Configuración → Aplicacións** desinstálaa.

Ou executa `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Instálase para
todos os usuarios en *Program Files* e engade unha entrada ao menú Inicio.
- **Actualizar:** executa o instalador máis novo. Substitúe a versión
  instalada.
- **Desinstalar:** usa **Configuración → Aplicacións**.
- **Instalador sen asinar:** se a versión non está asinada, SmartScreen avisa
  dun editor descoñecido. Escolle **Máis información → Executar de todos
  modos**.

O arquivo `.zip` é unha alternativa portátil: extráeo e executa
`fauste-player.exe`.

### macOS {#macos}

Abre `fauste-player-<version>-macos-universal.dmg` e arrastra **Fauste
Player** a **Aplicacións**. A mesma aplicación funciona en Apple silicon e
Intel (macOS 11 ou posterior).
- **Actualizar:** substitúe a aplicación do mesmo xeito.
- **Desinstalar:** móvea ao Lixo.
- **Aplicación sen asinar:** se a versión non está asinada e notarizada,
  macOS rexeita o primeiro arranque.
  - macOS 15 e posteriores: abre **Axustes do Sistema → Privacidade e
    seguridade**, desprázate ata a mensaxe sobre Fauste Player, escolle
    **Abrir igualmente** e confirma.
  - macOS 14 e anteriores: fai clic dereito na aplicación, escolle **Abrir**
    e confirma.
  - Ou executa `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK en macOS:** unha aplicación asinada e notarizada só pode cargar unha
  biblioteca JACK que estea asinada. Se non, JACK aparece como non
  dispoñible; usa Core Audio.
- **Listas:** en macOS impórtanse desde Configuración → Listas. Abrir un
  ficheiro de lista coa aplicación desde Finder non está admitido.

### Unha soa instancia á vez {#one-instance-at-a-time}

Só se executa un Fauste Player por cartafol de datos. Se abres unha lista
desde o xestor de ficheiros mentres se está executando, esa lista impórtase
na aplicación en execución. Se o inicias de novo sen ningunha lista, móstrase
unha mensaxe de que xa está en execución. Para executar instancias separadas
ao carón (por exemplo dous estudos nun mesmo computador), dálle a cada unha o
seu propio cartafol con `FAUSTE_HOME`.

### Liña de ordes {#command-line}

`fauste-player --version` imprime a versión. A barra de título da xanela
mostra o nome e a versión, e o botón **Acerca de** (unha icona de información,
á esquerda de **Configuración**) abre **Acerca de**, co autor, o copyright, a
licenza (GNU GPL versión 3 ou posterior; **Ler a licenza en liña** abre o seu
texto), un aviso breve e os avisos de licenza (**Licenzas de terceiros** abre
o ficheiro de avisos instalado cos paquetes de publicación). Péchase con
**Pechar** ou con `Esc`. `fauste-player --help` lista as opcións. Os ficheiros de lista pasados como
argumentos impórtanse como listas novas.

![A xanela Acerca de: versión, copyright e as licenzas dos compoñentes incluídos](../../images/guide/about.png)

En Windows o programa de publicación non abre ningunha xanela de consola:
`--version`, `--help` e os erros de inicio aparecen nun cadro de mensaxe.

Cando un cambio nas saídas agarda por algo que está en antena, aparece na
barra superior unha pílula **Axustes pendentes**; di o que agarda (consulta
[Configuración](settings.md#settings-pending)).

### Pechar con audio en antena {#closing-while-audio-is-on-air}

Pechar a xanela mentres hai algo en antena non sae da aplicación. A xanela
ponse diante (aínda que estivese minimizada) e un diálogo **Hai audio en
antena** lista o que está a soar: os reprodutores que están a reproducir ou
en pausa (`P1 — título`) e os cartuchos que soan co seu número na páxina
(`Cartucho 3 — título`). Un CUE de reprodutor ou de cartucheira non conta.
Escolle **Cancelar** (ou `Esc`, ou fai clic fóra do diálogo) para seguir
reproducindo, ou **Deter e pechar** para parar todos os reprodutores en
antena e todos os cartuchos e despois saír. A sesión gárdase como en
calquera outra saída. Se non hai nada en antena, a xanela péchase ao
momento. O diálogo ten prioridade sobre Configuración e Acerca de, e os
atallos de teclado non fan nada mentres está aberto (as ordes MIDI e remotas
si actúan).

## Primeiro arranque {#first-start}

A xanela ábrese con catro reprodutores, cada un cunha lista baleira. Non soa
nada ata que premes Play: isto tamén vale tras un reinicio ou un fallo.

## Engadir música {#add-music}

- Fai clic en **+ Engadir** na parte inferior dun reprodutor e escolle
  ficheiros, ou
- arrastra ficheiros de audio ou cartafoles desde o xestor de ficheiros a
  unha lista de pistas. Os cartafoles engaden os ficheiros de audio que
  teñen directamente dentro, non os seus subcartafoles.

Formatos admitidos: WAV, AIFF, CAF, FLAC, MP3 (e MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF e DFF) e audio Matroska (MKA).
Os ficheiros DSD reprodúcense convertidos a PCM á súa frecuencia dividida
por 32 (88,2 kHz para DSD64), ou sen cambios como DoP ou DSD nativo nun
dispositivo bit-perfect configurado para iso (consulta
[Saída bit-perfect](bit-perfect.md#dsd)). Opus sempre se reproduce a 48 kHz.
Os ficheiros WavPack deben ser sen perdas e mono ou estéreo; o WavPack
híbrido e o multicanle aparecen como ilexibles, igual que os ficheiros DFF
comprimidos con DST. Un WAV ou FLAC que contén un fluxo de bits DTS (ou outro
son envolvente codificado) en lugar de audio, como nalgunhas extraccións de
CD DTS, tamén aparece como ilexible (o rexistro di por que): convértoo
primeiro a PCM normal.

Os ficheiros envolventes (quad, 5.1, 6.1, 7.1 e outras disposicións de ata
oito canles) reprodúcense mesturados a estéreo segundo a posición de cada
canle: o frontal esquerdo e o dereito quedan no seu lado, o central e o
traseiro central van aos dous lados, as canles laterais e traseiras van ao
seu lado, e a canle LFE (subwoofer) descártase. Os ficheiros Opus e AAC
multicanle non se poden ler e aparecen como ilexibles.

Cada ficheiro analízase en segundo plano. A análise le o título, o artista,
o álbum e a portada, debuxa a forma de onda e atopa onde empeza e remata o
son e onde mesturar coa pista seguinte. Podes reproducir unha pista antes de
que remate a súa análise. Mentres se analizan ficheiros, a dereita da barra
de estado di cantos (**Analizando 12 pistas**); pon o punteiro enriba para ver
os ficheiros que se están analizando agora, cantos están en espera e, durante
un **Volver analizar todas as pistas**, canto levan feito. A lenda de cores
está á súa dereita; cando a xanela é estreita, ocúltanse primeiro elementos
da lenda e despois este elemento, así que un aviso á esquerda da barra de
estado segue lexible. Mentres só se executan medicións de sonoridade (consulta
[Sonoridade](loudness.md)) di **Medindo a sonoridade de N pistas**.

## Reproducir {#play}

- Preme **Play** (ou a tecla numérica do reprodutor, `1` para P1) para
  iniciar a pista **seguinte**, marcada en verde na lista.
- Preme **Play** de novo mentres unha pista está en antena para pasar con
  fundido á seguinte.
- Fai **dobre clic** nunha pista para marcala como seguinte.

No modo **CONT** (continuo) o reprodutor mestura coa pista seguinte por si só
no punto MIX. No modo **SINGLE** detense ao final de cada pista.
Continúa con [Reprodutores](players.md).

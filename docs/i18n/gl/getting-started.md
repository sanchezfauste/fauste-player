# Primeiros pasos

## Instalación {#install}

Descarga un paquete para o teu sistema da páxina **Releases** do proxecto.
Cada ficheiro ten un `.sha256` ao carón. Para comprobar unha descarga,
executa `sha256sum -c <file>.sha256` en Linux, ou
`shasum -a 256 -c <file>.sha256` en macOS.

### Linux {#linux}

| Paquete | Instalar | Actualizar | Desinstalar |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instala o `.deb` máis novo do mesmo xeito | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instala o `.rpm` máis novo | `sudo dnf remove fauste-player` |
| AppImage (calquera distribución) | `chmod +x fauste-player-<version>-x86_64.AppImage` e despois execútao | substitúe o ficheiro | elimina o ficheiro |
| Paquete Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instala o paquete máis novo | `flatpak uninstall org.fauste.FaustePlayer` |
| Arquivo | extráeo e executa `fauste-player` | extrae o máis novo | elimina o cartafol |

- **Integración co escritorio:** os paquetes engaden **Fauste Player** ao
  menú de aplicacións e permiten abrir con el listas `.m3u`, `.m3u8` e `.pls`,
  que se importan como listas novas.
- **Bibliotecas necesarias:** as bibliotecas ALSA e D-Bus. Todos os
  escritorios as teñen, e os paquetes decláranas. JACK úsase cando está
  instalado, e nunca é obrigatorio.
- **AppImage:** se non arranca porque falta FUSE, execútao con
  `--appimage-extract-and-run`.
- **Flatpak:**
  - O paquete necesita o runtime de freedesktop de Flathub. Se o remoto de
    Flathub non está configurado, executa primeiro
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - O sandbox pode ler o teu cartafol persoal (para reproducir a túa música
    onde está).
  - Reproduce a través de PulseAudio, ou directamente por ALSA para os
    dispositivos bit-perfect.
- **Bibliotecas do escritorio:** a xanela usa libxkbcommon e EGL ou OpenGL,
  con Wayland ou X11. Todos os escritorios as teñen, e o `.deb` e o `.rpm`
  decláranas. Nun sistema moi mínimo, instálaas antes de usar o AppImage.

### Windows {#windows}

Executa `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Instálase para
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
á esquerda de **Configuración**) abre **Acerca de**, co copyright e os avisos
de licenza (**Licenzas de terceiros** abre o ficheiro de avisos instalado cos
paquetes de publicación). Nun idioma traducido con IA, Acerca de tamén di que
a tradución pode conter erros. Péchase con **Pechar** ou con `Esc`.
`fauste-player --help` lista as opcións. Os ficheiros de lista pasados como
argumentos impórtanse como listas novas.

![A xanela Acerca de: versión, copyright e as licenzas dos compoñentes incluídos](../../images/guide/about.png)

En Windows o programa de publicación non abre ningunha xanela de consola:
`--version`, `--help` e os erros de inicio aparecen nun cadro de mensaxe.

Cando un axuste necesita un reinicio, aparece na barra superior unha pílula
**Reinicio pendente**: preme nela para reiniciar (consulta
[Configuración](settings.md#restart-pending)).

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
comprimidos con DST.

Cada ficheiro analízase en segundo plano. A análise le o título, o artista,
o álbum e a portada, debuxa a forma de onda e atopa onde empeza e remata o
son e onde mesturar coa pista seguinte. Podes reproducir unha pista antes de
que remate a súa análise.

## Reproducir {#play}

- Preme **Play** (ou a tecla numérica do reprodutor, `1` para P1) para
  iniciar a pista **seguinte**, marcada en verde na lista.
- Preme **Play** de novo mentres unha pista está en antena para pasar con
  fundido á seguinte.
- Fai **dobre clic** nunha pista para marcala como seguinte.

No modo **CONT** (continuo) o reprodutor mestura coa pista seguinte por si só
no punto MIX. No modo **SINGLE** detense ao final de cada pista.
Continúa con [Reprodutores](players.md).

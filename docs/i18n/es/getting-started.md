# Primeros pasos

## Instalación {#install}

En Windows, Fauste Player está en la **Microsoft Store**
(<https://apps.microsoft.com/detail/9NH4KCL73463>), y en Linux en la
**Snap Store** (<https://snapcraft.io/fauste-player>). O descarga un paquete
para tu sistema desde la página **Releases** del proyecto. Cada archivo tiene al lado un `.sha256`. Para comprobar una
descarga, ejecuta `sha256sum -c <file>.sha256` en Linux, o
`shasum -a 256 -c <file>.sha256` en macOS.

### Linux {#linux}

| Paquete | Instalar | Actualizar | Desinstalar |
|---|---|---|---|
| Snap (Snap Store) | `sudo snap install fauste-player` | automática (`sudo snap refresh fauste-player` para actualizar ahora) | `sudo snap remove fauste-player` |
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instala el `.deb` más nuevo de la misma forma | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instala el `.rpm` más nuevo | `sudo dnf remove fauste-player` |
| AppImage (cualquier distribución) | `chmod +x fauste-player-<version>-x86_64.AppImage` y ejecútalo | sustituye el archivo | borra el archivo |
| Paquete Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instala el paquete más nuevo | `flatpak uninstall com.sanchezfauste.FaustePlayer` |
| Archivo comprimido | extráelo y ejecuta `fauste-player` | extrae el más nuevo | borra la carpeta |

- **Integración con el escritorio:** los paquetes añaden **Fauste Player**
  al menú de aplicaciones y permiten abrir con él playlists `.m3u`, `.m3u8`
  y `.pls`, que se importan como playlists nuevas.
- **Bibliotecas necesarias:** las bibliotecas de ALSA y D-Bus. Todos los
  escritorios las tienen, y los paquetes las declaran. JACK se usa cuando
  está instalado, y nunca es obligatorio.
- **AppImage:** si no arranca porque falta FUSE, ejecútalo con
  `--appimage-extract-and-run`.
- **Snap:** el snap se ejecuta en un sandbox. De fábrica lee tu carpeta
  personal (salvo los archivos y carpetas ocultos), suena a través de
  PulseAudio o PipeWire y puede usar la red para la API de control remoto y
  OSC (ambos desactivados hasta que los actives). Dos funciones necesitan un
  permiso que das una vez, desde una terminal:
  - Salida bit-perfect en dispositivos `hw:` de ALSA, y superficies de
    control MIDI: `sudo snap connect fauste-player:alsa`.
  - Música en unidades USB y otros discos (`/media`, `/run/media`, `/mnt`):
    `sudo snap connect fauste-player:removable-media`.

  `snap connections fauste-player` muestra qué está conectado. Reinicia
  Fauste Player tras conectar. El snap no puede leer la música que esté en
  otros sitios (fuera de tu carpeta personal y esos tres): usa otro paquete
  para ella. JACK no está disponible en el snap. Sus ajustes y playlists se
  guardan en `~/snap/fauste-player/` (véase [Datos y copias de
  seguridad](data-and-backups.md)).
- **Flatpak:**
  - El paquete necesita el runtime de freedesktop de Flathub. Si el remoto
    de Flathub no está configurado, ejecuta antes
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - El sandbox puede leer tu carpeta personal (para reproducir tu música
    donde está).
  - Suena a través de PulseAudio, o directamente a través de ALSA para los
    dispositivos bit-perfect.
  - Puede usar la red, para la API de control remoto y OSC (ambos
    desactivados hasta que los actives).
  - Los paquetes hasta la 0.10 se instalaban como `org.fauste.FaustePlayer`, y
    los más nuevos se instalan como `com.sanchezfauste.FaustePlayer`, una
    aplicación aparte. Para conservar tus ajustes, playlists y cartuchos,
    cierra la aplicación y ejecuta
    `mv ~/.var/app/org.fauste.FaustePlayer ~/.var/app/com.sanchezfauste.FaustePlayer`
    antes de iniciar la nueva, y luego `flatpak uninstall org.fauste.FaustePlayer`.
- **Bibliotecas de escritorio:** la ventana usa libxkbcommon y EGL u OpenGL,
  con Wayland o X11. Todos los escritorios las tienen, y el `.deb` y el
  `.rpm` las declaran. En un sistema muy mínimo, instálalas antes de usar
  el AppImage.

### Windows {#windows}

Lo más fácil es la **Microsoft Store**: busca *Fauste Player*, o abre
<https://apps.microsoft.com/detail/9NH4KCL73463>, y elige **Obtener**. La
Store la mantiene al día, y **Configuración → Aplicaciones** la desinstala.

O ejecuta `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Se instala para
todos los usuarios en *Archivos de programa* y añade una entrada en el menú
Inicio.
- **Actualizar:** ejecuta el instalador más nuevo. Sustituye la versión
  instalada.
- **Desinstalar:** usa **Configuración → Aplicaciones**.
- **Instalador sin firmar:** si la release no está firmada, SmartScreen
  avisa de un editor desconocido. Elige **Más información → Ejecutar de
  todas formas**.

El archivo `.zip` es una alternativa portátil: extráelo y ejecuta
`fauste-player.exe`.

### macOS {#macos}

Abre `fauste-player-<version>-macos-universal.dmg` y arrastra **Fauste
Player** a **Aplicaciones**. La misma aplicación funciona en Apple silicon e
Intel (macOS 11 o posterior).
- **Actualizar:** sustituye la aplicación de la misma forma.
- **Desinstalar:** muévela a la Papelera.
- **Aplicación sin firmar:** si la release no está firmada y notarizada,
  macOS rechaza el primer arranque.
  - macOS 15 y posteriores: abre **Ajustes del Sistema → Privacidad y
    seguridad**, desplázate hasta el mensaje sobre Fauste Player, elige
    **Abrir igualmente** y confirma.
  - macOS 14 y anteriores: haz clic derecho en la aplicación, elige
    **Abrir** y confirma.
  - O ejecuta `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK en macOS:** una aplicación firmada y notarizada solo puede cargar
  una biblioteca de JACK que también esté firmada. Si no, JACK aparece como
  no disponible; usa Core Audio.
- **Playlists:** en macOS se importan desde Configuración → Playlists. No se
  admite abrir un archivo de playlist con la aplicación desde el Finder.

### Una sola instancia a la vez {#one-instance-at-a-time}

Solo se ejecuta un Fauste Player por carpeta de datos. Abrir una playlist
desde el gestor de archivos mientras está en marcha la importa en la
aplicación que ya se está ejecutando. Volver a arrancarlo sin ninguna
playlist muestra un mensaje de que ya está en marcha. Para ejecutar
instancias separadas a la vez (por ejemplo, dos estudios en un mismo
ordenador), dale a cada una su propia carpeta con `FAUSTE_HOME`.

### Línea de comandos {#command-line}

`fauste-player --version` muestra la versión. La barra de título de la
ventana muestra el nombre y la versión, y el botón **Acerca de** (un icono de
información, a la izquierda de **Configuración**) abre la ventana **Acerca
de**, con el autor, el copyright, la licencia (GNU GPL versión 3 o
posterior; **Leer la licencia en línea** abre su texto), un breve aviso
legal y los avisos de licencia (**Licencias de terceros** abre el archivo de
avisos que se instala con los paquetes de release). Ciérrala con **Cerrar**
o con `Esc`.
`fauste-player --help` lista las opciones. Los archivos de playlist que se
pasan como argumentos se importan como playlists nuevas.

![La ventana Acerca de: versión, copyright y las licencias de los componentes incluidos](../../images/guide/about.png)

En Windows, el programa de release no abre ninguna ventana de consola:
`--version`, `--help` y los errores de arranque aparecen en un cuadro de
mensaje.

Cuando un cambio en las salidas espera a algo que está en el aire, aparece
en la barra superior la etiqueta **Ajustes pendientes**; dice qué espera
(consulta [Configuración](settings.md#settings-pending)).

### Cerrar con audio al aire {#closing-while-audio-is-on-air}

Cerrar la ventana mientras algo suena al aire no cierra la aplicación. La
ventana pasa al frente (aunque estuviera minimizada) y un diálogo **Hay audio
en el aire** lista lo que está sonando: los players que están reproduciendo
o en pausa (`P1 — título`) y los cartuchos que suenan con su número en la
página (`Cartucho 3 — título`). Un CUE de un player o de la cartuchera no
cuenta. Elige **Cancelar** (o `Esc`, o haz clic fuera del diálogo) para
seguir sonando, o **Detener y cerrar** para detener todos los players al
aire y todos los cartuchos y luego salir. La sesión se guarda como en
cualquier otra salida. Si no hay nada al aire, la ventana se cierra en el
acto. El diálogo tiene prioridad sobre Configuración y Acerca de, y los
atajos de teclado no hacen nada mientras está abierto (los comandos MIDI y
remotos siguen funcionando).

## Primer arranque {#first-start}

La ventana se abre con cuatro players, cada uno con una playlist vacía. No
suena nada hasta que pulsas Play: también después de un reinicio o de un
fallo.

## Añadir música {#add-music}

- Haz clic en **+ Añadir** en la parte de abajo de un player y elige
  archivos, o
- arrastra archivos o carpetas de audio desde tu gestor de archivos a una
  lista de pistas. Las carpetas añaden los archivos de audio que hay
  directamente en ellas, no los de sus subcarpetas.

Formatos admitidos: WAV, AIFF, CAF, FLAC, MP3 (y MP1/MP2), AAC/M4A, ALAC, Ogg
Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF y DFF) y audio
Matroska (MKA). Los archivos DSD se reproducen convertidos a PCM a su
frecuencia dividida entre 32 (88,2 kHz para DSD64), o sin cambios como DoP o
DSD nativo en un dispositivo bit-perfect configurado para ello (consulta
[Salida bit-perfect](bit-perfect.md#dsd)). Opus siempre se reproduce a
48 kHz. Los archivos WavPack deben ser sin pérdida y mono o estéreo; los
WavPack híbridos y multicanal aparecen como ilegibles, igual que los DFF con
compresión DST. Un WAV o FLAC que contiene un flujo de bits DTS (u otro
sonido envolvente codificado) en lugar de audio, como ocurre con algunas
extracciones de CD DTS, también aparece como ilegible (el registro dice por
qué): conviértelo antes a PCM normal.

Los archivos envolventes (cuadrafónico, 5.1, 6.1, 7.1 y otras disposiciones
de hasta ocho canales) se reproducen mezclados a estéreo según la posición
de cada canal: el frontal izquierdo y el derecho se quedan en su lado, el
central y el trasero central van a los dos lados, los canales laterales y
traseros van a su lado, y el canal LFE (subwoofer) se descarta. Los
archivos Opus y AAC multicanal no se pueden leer y aparecen como ilegibles.

Cada archivo se analiza en segundo plano. El análisis lee el título, el
artista, el álbum y la carátula, dibuja la forma de onda y encuentra dónde
empieza y acaba el sonido y dónde mezclar con la siguiente pista. Puedes
reproducir una pista antes de que termine su análisis. Mientras se analizan
archivos, la derecha de la barra de estado dice cuántos (**Analizando 12
pistas**); pasa el cursor por encima para ver los archivos que se analizan
ahora, cuántos esperan y, durante un **Volver a analizar todas las pistas**,
cuánto lleva. La leyenda de colores está a su derecha; cuando la ventana es
estrecha, se ocultan primero los elementos de la leyenda y luego este
elemento, para que un aviso a la izquierda de la barra de estado siga siendo
legible. Mientras solo se ejecutan mediciones de sonoridad (véase
[Sonoridad](loudness.md)) dice **Midiendo la sonoridad de N pistas**.

## Reproducir {#play}

- Pulsa **Play** (o la tecla numérica del player, `1` para P1) para iniciar
  la pista **siguiente**, marcada en verde en la lista.
- Vuelve a pulsar **Play** mientras una pista suena para hacer un fundido
  hacia la siguiente.
- Haz **doble clic** en una pista para convertirla en la siguiente.

En modo **CONT** (continuo), el player mezcla con la siguiente pista por sí
solo en el punto MIX. En modo **SINGLE** se detiene al final de cada pista.
Sigue con [Players](players.md).

# Primeros pasos

## Instalación {#install}

Descarga un paquete para tu sistema desde la página **Releases** del
proyecto. Cada archivo tiene al lado un `.sha256`. Para comprobar una
descarga, ejecuta `sha256sum -c <file>.sha256` en Linux, o
`shasum -a 256 -c <file>.sha256` en macOS.

### Linux {#linux}

| Paquete | Instalar | Actualizar | Desinstalar |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instala el `.deb` más nuevo de la misma forma | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instala el `.rpm` más nuevo | `sudo dnf remove fauste-player` |
| AppImage (cualquier distribución) | `chmod +x fauste-player-<version>-x86_64.AppImage` y ejecútalo | sustituye el archivo | borra el archivo |
| Paquete Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instala el paquete más nuevo | `flatpak uninstall org.fauste.FaustePlayer` |
| Archivo comprimido | extráelo y ejecuta `fauste-player` | extrae el más nuevo | borra la carpeta |

- **Integración con el escritorio:** los paquetes añaden **Fauste Player**
  al menú de aplicaciones y permiten abrir con él playlists `.m3u`, `.m3u8`
  y `.pls`, que se importan como playlists nuevas.
- **Bibliotecas necesarias:** las bibliotecas de ALSA y D-Bus. Todos los
  escritorios las tienen, y los paquetes las declaran. JACK se usa cuando
  está instalado, y nunca es obligatorio.
- **AppImage:** si no arranca porque falta FUSE, ejecútalo con
  `--appimage-extract-and-run`.
- **Flatpak:**
  - El paquete necesita el runtime de freedesktop de Flathub. Si el remoto
    de Flathub no está configurado, ejecuta antes
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - El sandbox puede leer tu carpeta personal (para reproducir tu música
    donde está).
  - Suena a través de PulseAudio, o directamente a través de ALSA para los
    dispositivos bit-perfect.
- **Bibliotecas de escritorio:** la ventana usa libxkbcommon y EGL u OpenGL,
  con Wayland o X11. Todos los escritorios las tienen, y el `.deb` y el
  `.rpm` las declaran. En un sistema muy mínimo, instálalas antes de usar
  el AppImage.

### Windows {#windows}

Ejecuta `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Se instala para
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
de**, con el copyright y los avisos de licencia (**Licencias de terceros**
abre el archivo de avisos que se instala con los paquetes de release). En un
idioma traducido con IA, Acerca de también indica que la traducción puede
contener errores. Ciérrala con **Cerrar** o con `Esc`.
`fauste-player --help` lista las opciones. Los archivos de playlist que se
pasan como argumentos se importan como playlists nuevas.

![La ventana Acerca de: versión, copyright y las licencias de los componentes incluidos](../../images/guide/about.png)

En Windows, el programa de release no abre ninguna ventana de consola:
`--version`, `--help` y los errores de arranque aparecen en un cuadro de
mensaje.

Cuando un ajuste necesita reiniciar, aparece en la barra superior la
etiqueta **Reinicio pendiente**: púlsala para reiniciar (consulta
[Configuración](settings.md#restart-pending)).

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
compresión DST.

Cada archivo se analiza en segundo plano. El análisis lee el título, el
artista, el álbum y la carátula, dibuja la forma de onda y encuentra dónde
empieza y acaba el sonido y dónde mezclar con la siguiente pista. Puedes
reproducir una pista antes de que termine su análisis.

## Reproducir {#play}

- Pulsa **Play** (o la tecla numérica del player, `1` para P1) para iniciar
  la pista **siguiente**, marcada en verde en la lista.
- Vuelve a pulsar **Play** mientras una pista suena para hacer un fundido
  hacia la siguiente.
- Haz **doble clic** en una pista para convertirla en la siguiente.

En modo **CONT** (continuo), el player mezcla con la siguiente pista por sí
solo en el punto MIX. En modo **SINGLE** se detiene al final de cada pista.
Sigue con [Players](players.md).

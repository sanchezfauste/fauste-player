# Primeiros passos

## Instalação {#install}

No Windows, o Fauste Player está na **Microsoft Store**
(<https://apps.microsoft.com/detail/9NH4KCL73463>) e, no Linux, na
**Snap Store** (<https://snapcraft.io/fauste-player>). Ou descarregue um
pacote para o seu sistema a partir da página **Releases** do projeto. Cada ficheiro tem ao lado um `.sha256`. Para verificar uma
transferência, execute `sha256sum -c <ficheiro>.sha256` no Linux, ou
`shasum -a 256 -c <ficheiro>.sha256` no macOS.

### Linux {#linux}

| Pacote | Instalar | Atualizar | Remover |
|---|---|---|---|
| Snap (Snap Store) | `sudo snap install fauste-player` | automática (`sudo snap refresh fauste-player` para atualizar já) | `sudo snap remove fauste-player` |
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instale o `.deb` mais recente da mesma forma | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instale o `.rpm` mais recente | `sudo dnf remove fauste-player` |
| AppImage (qualquer distribuição) | `chmod +x fauste-player-<version>-x86_64.AppImage` e execute-o | substitua o ficheiro | apague o ficheiro |
| Pacote Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instale o pacote mais recente | `flatpak uninstall com.sanchezfauste.FaustePlayer` |
| Arquivo | extraia-o e execute `fauste-player` | extraia o mais recente | apague a pasta |

- **Integração no ambiente de trabalho:** os pacotes adicionam o **Fauste
  Player** ao menu de aplicações e permitem abrir com ele listas `.m3u`,
  `.m3u8` e `.pls`, que são importadas como listas novas.
- **Bibliotecas necessárias:** as bibliotecas ALSA e D-Bus. Todos os
  ambientes de trabalho as têm, e os pacotes declaram-nas. O JACK é usado
  quando está instalado e nunca é obrigatório.
- **AppImage:** se não arrancar por falta do FUSE, execute-o com
  `--appimage-extract-and-run`.
- **Snap:** o snap corre numa sandbox. De origem, lê a sua pasta pessoal
  (exceto ficheiros e pastas ocultos), toca através do PulseAudio ou do
  PipeWire e pode usar a rede para a API de controlo remoto e o OSC (ambos
  desativados até os ativar). Duas funcionalidades precisam de uma permissão
  que dá uma só vez, a partir de um terminal:
  - Saída bit-perfect em dispositivos ALSA `hw:` e superfícies de controlo
    MIDI: `sudo snap connect fauste-player:alsa`.
  - Música em unidades USB e outros discos (`/media`, `/run/media`, `/mnt`):
    `sudo snap connect fauste-player:removable-media`.

  `snap connections fauste-player` mostra o que está ligado. Reinicie o
  Fauste Player depois de ligar. A música noutros locais (fora da sua pasta
  pessoal e destes três) não pode ser lida pelo snap: use outro pacote para
  ela. O JACK não está disponível no snap. As suas definições e listas ficam
  em `~/snap/fauste-player/` (ver
  [Dados e cópias de segurança](data-and-backups.md)).
- **Flatpak:**
  - O pacote precisa do runtime freedesktop do Flathub. Se o repositório
    remoto do Flathub não estiver configurado, execute primeiro
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - A sandbox pode ler a sua pasta pessoal (para tocar a sua música onde
    está).
  - Toca através do PulseAudio, ou diretamente através do ALSA para
    dispositivos bit-perfect.
  - Pode usar a rede, para a API de controlo remoto e o OSC (ambos
    desativados até os ativar).
  - Os pacotes até à 0.10 eram instalados como `org.fauste.FaustePlayer`, e
    os mais recentes instalam-se como `com.sanchezfauste.FaustePlayer`, uma
    aplicação separada. Para manter as suas definições, listas e cartuchos,
    feche a aplicação e execute
    `mv ~/.var/app/org.fauste.FaustePlayer ~/.var/app/com.sanchezfauste.FaustePlayer`
    antes de iniciar a nova, e depois `flatpak uninstall org.fauste.FaustePlayer`.
- **Bibliotecas do ambiente de trabalho:** a janela usa a libxkbcommon e o
  EGL ou OpenGL, com Wayland ou X11. Todos os ambientes de trabalho as têm, e
  o `.deb` e o `.rpm` declaram-nas. Num sistema muito minimalista, instale-as
  antes de usar o AppImage.

### Windows {#windows}

O mais fácil é a **Microsoft Store**: procure *Fauste Player*, ou abra
<https://apps.microsoft.com/detail/9NH4KCL73463>, e escolha **Obter**. A
Store mantém-no atualizado, e **Definições → Aplicações** remove-o.

Ou execute `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Instala para
todos os utilizadores em *Program Files* e adiciona uma entrada ao menu
Iniciar.
- **Atualizar:** execute o instalador mais recente. Substitui a versão
  instalada.
- **Remover:** use **Definições → Aplicações**.
- **Instalador não assinado:** se a versão não estiver assinada, o
  SmartScreen avisa de que o editor é desconhecido. Escolha **Mais
  informações → Executar mesmo assim**.

O arquivo `.zip` é uma alternativa portátil: extraia-o e execute
`fauste-player.exe`.

### macOS {#macos}

Abra `fauste-player-<version>-macos-universal.dmg` e arraste o **Fauste
Player** para **Aplicações**. A mesma aplicação funciona em Apple silicon e
Intel (macOS 11 ou posterior).
- **Atualizar:** substitua a aplicação da mesma forma.
- **Remover:** mova-a para a Reciclagem.
- **Aplicação não assinada:** se a versão não estiver assinada e
  notarizada, o macOS recusa o primeiro arranque.
  - macOS 15 e posterior: abra **Definições do Sistema → Privacidade e
    Segurança**, desloque-se até à mensagem sobre o Fauste Player, escolha
    **Abrir Mesmo Assim** e confirme.
  - macOS 14 e anterior: clique com o botão direito na aplicação, escolha
    **Abrir** e confirme.
  - Ou execute `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK no macOS:** uma aplicação assinada e notarizada só pode carregar uma
  biblioteca JACK que também esteja assinada. Caso contrário, o JACK aparece
  como indisponível; use o Core Audio.
- **Listas:** no macOS são importadas em Definições → Listas. Abrir um
  ficheiro de lista com a aplicação a partir do Finder não é suportado.

### Uma só instância de cada vez {#one-instance-at-a-time}

Só corre um Fauste Player por pasta de dados. Abrir uma lista a partir do
gestor de ficheiros enquanto a aplicação está em execução importa essa lista
para a aplicação em execução. Iniciá-la de novo sem nenhuma lista mostra uma
mensagem a dizer que já está em execução. Para executar instâncias separadas
lado a lado (por exemplo, dois estúdios num só computador), dê a cada uma a
sua própria pasta com `FAUSTE_HOME`.

### Linha de comandos {#command-line}

`fauste-player --version` imprime a versão. A barra de título da janela
mostra o nome e a versão, e o botão **Acerca do Fauste Player** (um ícone de
informação, à esquerda de **Definições**) abre a janela de informação, com o
autor, o copyright, a licença (GNU GPL versão 3 ou posterior; **Ler a licença
em linha** abre o seu texto), um aviso legal breve e os avisos de licença
(**Licenças de terceiros** abre o ficheiro de avisos instalado com os
pacotes de lançamento). Feche-a com **Fechar** ou `Esc`. `fauste-player --help` lista as opções. Os ficheiros de
lista passados como argumentos são importados como listas novas.

![A janela Acerca de: versão, copyright e licenças dos componentes incluídos](../../images/guide/about.png)

No Windows, o programa de lançamento não abre nenhuma janela de consola:
`--version`, `--help` e os erros de arranque aparecem numa caixa de mensagem.

Quando uma alteração das saídas espera por algo que está no ar, aparece uma
etiqueta **Definições pendentes** na barra superior; indica o que está à
espera (ver [Definições](settings.md#settings-pending)).

### Fechar com áudio no ar {#closing-while-audio-is-on-air}

Fechar a janela enquanto algo está no ar não encerra a aplicação. A janela
passa para primeiro plano (mesmo que estivesse minimizada) e um diálogo **Há
áudio no ar** lista o que está a soar: os leitores que estão a tocar ou em
pausa (`P1 — título`) e os cartuchos a tocar com o seu número na página
(`Cartucho 3 — título`). O CUE de um leitor ou da cartucheira não conta.
Escolha **Cancelar** (ou `Esc`, ou clique fora do diálogo) para continuar a
tocar, ou **Parar e fechar** para parar todos os leitores no ar e todos os
cartuchos e depois sair. A sessão é guardada como em qualquer outra saída.
Sem nada no ar, a janela fecha de imediato. O diálogo tem precedência sobre
as Definições e sobre a janela de informação, e os atalhos de teclado não
fazem nada enquanto está aberto (os comandos MIDI e remotos continuam a
atuar).

## Primeiro arranque {#first-start}

A janela abre com quatro leitores, cada um com uma lista vazia. Nada toca
até premir Play: isto também é verdade após um reinício ou uma falha.

## Adicionar música {#add-music}

- Clique em **+ Adicionar** na parte inferior de um leitor e escolha
  ficheiros, ou
- arraste ficheiros de áudio ou pastas do gestor de ficheiros para uma lista
  de faixas. As pastas adicionam os ficheiros de áudio que contêm
  diretamente, não as suas subpastas.

Formatos suportados: WAV, AIFF, CAF, FLAC, MP3 (e MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF e DFF) e áudio Matroska (MKA).
Os ficheiros DSD tocam convertidos para PCM à sua frequência dividida por 32
(88,2 kHz para DSD64), ou sem alterações como DoP ou DSD nativo num
dispositivo bit-perfect configurado para isso (ver
[Saída bit-perfect](bit-perfect.md#dsd)). O Opus toca sempre a 48 kHz. Os
ficheiros WavPack têm de ser sem perdas e mono ou estéreo; os WavPack
híbridos e multicanal aparecem como ilegíveis, tal como os ficheiros DFF
comprimidos com DST. Um WAV ou FLAC que contém um fluxo de bits DTS (ou
outro som envolvente codificado) em vez de áudio, como alguns rips de CD DTS,
também aparece como ilegível (o registo diz porquê): converta-o primeiro para
PCM normal.

Os ficheiros envolventes (quad, 5.1, 6.1, 7.1 e outras disposições até oito
canais) tocam misturados para estéreo segundo a posição de cada canal: a
frente esquerda e direita ficam do seu lado, o centro e o centro traseiro vão
para ambos os lados, os canais laterais e traseiros vão para o seu lado, e o
canal LFE (subwoofer) é descartado. Os ficheiros Opus e AAC multicanal não
podem ser lidos e aparecem como ilegíveis.

Cada ficheiro é analisado em segundo plano. A análise lê o título, o
artista, o álbum e a capa, desenha a forma de onda e encontra onde o som
começa e acaba e onde misturar com a faixa seguinte. Pode tocar uma faixa
antes de a sua análise terminar. Enquanto há ficheiros a ser analisados, a
direita da barra de estado diz quantos (**A analisar 12 faixas**); passe o
rato por cima para ver os ficheiros que estão a ser analisados agora, quantos
estão à espera e, durante um **Voltar a analisar todas as faixas**, até onde
chegou. A legenda de cores fica à sua direita; quando a janela é estreita,
primeiro desaparecem os itens da legenda e depois este item, para que um
aviso à esquerda da barra de estado continue legível. Enquanto só correm
medições de loudness (ver [Loudness](loudness.md)) diz **A medir a loudness
de N faixas**.

## Reproduzir {#play}

- Prima **Play** (ou a tecla numérica do leitor, `1` para o P1) para iniciar
  a faixa **seguinte**, marcada a verde na lista.
- Prima **Play** de novo enquanto uma faixa está no ar para fazer fade para
  a seguinte.
- Faça **duplo clique** numa faixa para a tornar a seguinte.

No modo **CONT** (contínuo), o leitor mistura com a faixa seguinte por si só
no ponto MIX. No modo **SINGLE**, para no fim de cada faixa. Continue com
[Leitores](players.md).

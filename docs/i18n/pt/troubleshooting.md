# Resolução de problemas

## Sem som {#no-sound}

1. Abra **Definições → Saídas de áudio** e prima **Testar Main** para o
   leitor. Se ouvir o tom, verifique o fader de volume do leitor.
2. Se não ouvir nada, escolha outro dispositivo ou par de canais. Uma
   alteração das saídas aplica-se quando nada toca nelas; **Aplicar agora**
   nas Definições aplica-a de imediato.
3. No Linux, prefira **PipeWire** ou **PulseAudio** em Definições → Saídas de
   áudio → Sistema de áudio. Partilham a placa de som com outros programas. O
   **ALSA** fala diretamente com a placa e pode encontrá-la ocupada.
4. O **JACK** aparece como indisponível («no output device») quando nenhum
   servidor JACK está em execução. Inicie o servidor (por exemplo, com o
   QjackCtl) e reinicie a aplicação. Defina o servidor JACK para a frequência
   de amostragem das Definições (48 kHz por predefinição): o JACK corre a uma
   só frequência para todos os programas.
5. O **PipeWire** não é oferecido pelos arquivos descarregáveis; estes
   chegam ao PipeWire através do seu serviço PulseAudio, que funciona da
   mesma forma. Está disponível nas versões compiladas com a funcionalidade
   `pipewire`.

## Bit-perfect {#bit-perfect}

- **O indicador BP fica apagado.** Verifique cada condição em
  [Saída bit-perfect](bit-perfect.md#the-bp-badge): nenhum fade, nada
  mais nas mesmas saídas, um ficheiro sem perdas que tenha sido analisado e
  um dispositivo a correr à frequência do ficheiro.
- **Um breve silêncio antes de uma faixa.** O dispositivo bit-perfect foi
  aberto de novo à frequência de amostragem da faixa. Só acontece quando a
  frequência muda: as faixas à mesma frequência seguem-se sem pausa. Mantenha
  a biblioteca a uma só frequência para o evitar. Ver
  [O que custa](bit-perfect.md#what-it-costs).
- **As faixas não se misturam, e o Play não faz fade.** Num leitor cuja saída
  Main é bit-perfect, cada faixa toca até ao fim e só então começa a
  seguinte, e o Play / Seguinte e o Anterior mudam de imediato: não há
  crossfades nem segues nesse leitor. Desative o bit-perfect do dispositivo,
  ou mude o leitor para outra saída, para voltar a misturar. Ver
  [O que custa](bit-perfect.md#what-it-costs).
- **O fader de volume não se move.** A saída Main do leitor é bit-perfect, o
  que mantém o volume a 100 %; a dica do fader di-lo. Os faders MIDI e a API
  remota também não o conseguem mover. Ver
  [O que custa](bit-perfect.md#what-it-costs).
- **Uma faixa toca reamostrada, e o registo diz que o dispositivo está
  ocupado (Linux).** Para mudar de frequência, a aplicação fecha o
  dispositivo e abre-o de novo. Nesse momento, o servidor de som (PipeWire)
  pode apoderar-se da placa. A aplicação tenta de novo algumas vezes; se a
  placa continuar ocupada, a faixa toca à frequência atual do dispositivo, e
  a faixa seguinte pede de novo a sua frequência. Para dar a placa só à
  aplicação, abra as definições de som do sistema e ponha o perfil dessa
  placa em **Desligado** (ou **Pro Audio**), para que o servidor de som deixe
  em paz o seu dispositivo `hw:`. O número de tentativas e a espera entre
  elas são `tuning.device_busy_retries` e `tuning.device_busy_retry_ms` no
  ficheiro de configuração.
- **O dispositivo toca, mas o indicador BP fica apagado (Windows ou
  macOS).** O acesso exclusivo foi recusado, e o dispositivo toca
  partilhado.
  - Windows: outro programa pode ter o dispositivo em exclusivo, ou o
    controlo exclusivo está desativado nas propriedades Avançadas do
    dispositivo.
  - macOS: outro programa pode ter o dispositivo em hog mode, ou o
    dispositivo oferece as suas frequências apenas como um intervalo contínuo
    (a maioria das interfaces lista frequências fixas).
- **Um dispositivo `hw:` não pode ser aberto (Linux).**
  - Um servidor de som pode estar a segurar a placa. Pare-o, ou configure o
    servidor para deixar essa placa em paz, e reinicie a aplicação.
  - Alguns DAC USB só aceitam amostras de 24 bits empacotadas (`S24_3LE`), que
    a biblioteca de áudio não suporta. Use essa placa através de `plughw:`
    (não é bit-perfect).

### DSD {#dsd}

- **Uma faixa DSD toca convertida embora o dispositivo esteja em DoP ou DSD
  nativo.** O registo diz porquê («DSD converted to PCM» e o motivo) nestas
  causas: o volume do leitor não está a 100 %, algo mais a tocar no
  dispositivo, mais de dois canais, ou um dispositivo que recusa a frequência
  (o DoP precisa da frequência DSD dividida por 16, por exemplo 176,4 kHz para
  DSD64) ou não tem formato de 24 ou 32 bits. Uma faixa que ainda não foi
  analisada converte em silêncio, sem linha no registo: analise-a (Definições
  → Análise) e toque-a de novo.
- **Só a primeira faixa de um álbum DSD sai como DSD.** Essa é a definição
  de mistura predefinida: as faixas que o leitor inicia por si só tocam
  convertidas. Escolha **Manter o DSD e silenciar as outras fontes** em
  Definições → Saídas de áudio para as manter em DSD. Ver
  [DSD](bit-perfect.md#dsd).
- **O cabeçalho mostra DSD mas o conversor toca ruído ou não sincroniza.** O
  conversor não reconhece o DoP (ou o formato nativo). Volte a pôr o
  dispositivo em **Converter para PCM**.
- **Um estalido quando uma faixa DSD começa, para ou deixa o DSD.** O
  conversor precisa de mais silêncio DSD: aumente o **Silêncio DSD** (200 ms
  por predefinição) em Definições → Saídas de áudio, Avançado.
- **Outros leitores ou cartuchos estão silenciosos no dispositivo.** Uma
  faixa DSD está a tocar com **Manter o DSD e silenciar as outras fontes**; o
  indicador **Outras silenciadas** aparece. Voltam a soar quando a faixa
  termina.

## Aviso de «Saída perdida» {#output-lost-alert}

A barra de estado mostra **Saída perdida: &lt;dispositivo&gt;** quando um
dispositivo deixa de responder. Os leitores continuam a contar e a misturar
com um relógio interno, para que a automação não bloqueie. O dispositivo é
tentado de novo a cada 2 segundos e é retomado quando regressa. Volte a
ligar o cabo ou volte a ligar a interface.

### «Saída perdida» que nunca desaparece, com uma saída `hw:` direta {#output-lost-that-never-clears-with-a-direct-hw-output}

Uma placa de som usada através de uma saída ALSA `hw:` direta (por exemplo,
uma saída bit-perfect) é retida só pelo Fauste Player: o servidor de som
(PipeWire ou PulseAudio) não a pode usar ao mesmo tempo. Se outra saída passar
pelo dispositivo predefinido do servidor de som e esse dispositivo
predefinido for a mesma placa, essa saída nunca arranca e fica em **Saída
perdida**. O registo diz «output device opened but never started» uma vez.

Use um caminho por placa: encaminhe todas as saídas dessa placa pelo mesmo
dispositivo `hw:` (com canais diferentes, se necessário), ou escolha outra
placa como saída predefinida do servidor de som nas definições de som do
sistema.

## Uma faixa mostra um ícone de aviso ou um ficheiro com uma cruz {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

O ficheiro está em falta (movido, apagado, desmontado: um ficheiro com uma
cruz) ou não pode ser descodificado (um sinal de aviso). Os leitores
saltam-no. Passe o rato sobre a linha para ver qual dos casos é, e o
caminho do ficheiro.

Um ficheiro em falta é procurado de novo a cada 30 segundos
(`tuning.missing_recheck_ms` no ficheiro de configuração): quando a unidade é
montada ou o ficheiro é posto de volta, a faixa passa a poder tocar por si
só. Um ficheiro que não pode ser descodificado é verificado de novo por si
só no mesmo temporizador, por tamanho e data de modificação: não é
descodificado de novo a menos que um deles tenha mudado, por exemplo quando
uma cópia termina. Para o verificar de imediato, use **Voltar a analisar** no
menu da sua linha, ou **Definições → Análise → Voltar a analisar todas as
faixas** para toda a biblioteca.

Um WAV ou FLAC que aparece como ilegível, ou que tocaria como ruído alto,
pode ser um rip de CD DTS: o ficheiro contém um fluxo de bits DTS em vez de
áudio. A aplicação recusa-o (o registo diz «encoded bitstream stored as
PCM») e não descodifica DTS. Converta-o primeiro para PCM normal com outra
ferramenta.

## Uma pasta vigiada mostra um aviso {#a-watched-folder-shows-a-warning}

Uma categoria que vigia uma pasta mostra ⟳, com outra cor quando a pasta não
é seguida; passe o rato por cima para ver o motivo. Ver [Pastas
vigiadas](library.md#watched-folders).

- **Não encontrada ou não pode ser lida**: a unidade ou a partilha não está
  montada ou não está acessível. Uma pasta vazia que devia conter faixas da
  biblioteca também conta como não encontrada, pois uma partilha desmontada
  costuma parecer uma. Nada é marcado nem removido; a pasta é procurada de
  novo a cada 30 segundos e analisada quando aparece. Se esvaziou a pasta de
  propósito, limpe as suas faixas selecionando-as na tabela da biblioteca e
  escolhendo **Remover da biblioteca…**, ou escolhendo **Voltar a analisar**
  nelas, o que marca como em falta as que já não têm ficheiro para que
  **Remover faixas em falta…** as possa remover.
- **As alterações não são seguidas**: o sistema não a quis vigiar,
  normalmente uma pasta com muitíssimas subpastas. A pasta continua a ser
  analisada em cada arranque e em cada nova tentativa, por isso os ficheiros
  novos aparecem então, não de imediato. No Linux, uma árvore de pastas
  maior do que o limite de vigilâncias por utilizador precisa de um
  `fs.inotify.max_user_watches` mais alto (por exemplo `sudo sysctl
  fs.inotify.max_user_watches=524288`; torne-o permanente em
  `/etc/sysctl.d/`).
- **Não é um caminho completo**: a pasta guardada foi editada à mão para um
  caminho que não está completo. Escolha **Alterar pasta vigiada…** para a
  escolher de novo.
- **Uma partilha de rede não mostra aviso, mas os ficheiros adicionados a
  partir de outro computador não aparecem**: numa partilha de rede
  (SMB/CIFS, NFS) o sistema costuma aceitar a vigilância mas só comunica as
  alterações feitas a partir deste computador. As alterações feitas noutro
  sítio são vistas no arranque seguinte, não enquanto a aplicação corre.
- **Uma pasta com milhares de ficheiros** demora a ser analisada no arranque;
  o progresso da análise mostra-o.
- As faixas cujo ficheiro saiu de uma pasta vigiada ficam na biblioteca
  marcadas como em falta. Remova-as com **Remover faixas em falta…** no menu
  da categoria.

## Cortes de áudio {#audio-dropouts}

A barra de estado avisa durante 5 segundos após cada corte que a aplicação
deteta: **P1: cortes de áudio (3)** quando a descodificação de um leitor não
acompanhou o disco (a contagem é da faixa que está a tocar) e
**&lt;dispositivo&gt;: cortes do dispositivo de áudio (2)** quando o
dispositivo de saída falhou um prazo (um xrun). O registo também regista
cada um, no máximo uma linha a cada 10 segundos por tipo, com quantos
aconteceram. Nem todos os sistemas de áudio comunicam xruns (o PulseAudio
não; o modo exclusivo do Windows não).


- Aumente o **tamanho do buffer** nas Definições (aplica-se quando nada toca
  nesse dispositivo, ou de imediato com **Aplicar agora**).
- Através de um servidor de som (PulseAudio, o dispositivo `default` do
  ALSA), cada saída já pede pelo menos 50 ms seja qual for o tamanho do
  buffer, para que um computador ocupado não se ouça; as Definições mostram-no
  junto à latência. Num computador antigo ou muito carregado, aumente
  `shared_buffer_ms` na secção `outputs` do ficheiro de configuração (por
  exemplo para 100). O sistema mantém cerca de dois desses blocos em fila, por
  isso os botões de cartucho e de play passam a soar cerca de duas vezes o
  aumento mais tarde. Para a saída do ar, a entrada de hardware do próprio
  dispositivo (`hw:` no Linux) evita por completo o servidor de som.
- No Linux, permita o escalonamento em tempo real. A aplicação pede-o ao
  sistema através do rtkit (D-Bus), e o registo regista uma recusa. Muitos
  ambientes de trabalho recusam-no às aplicações.
- Evite unidades de rede para a música que toca no ar.

## A interface está lenta num computador antigo {#the-interface-is-slow-on-an-old-computer}

Enquanto algo toca, a interface desenha no máximo 30 imagens por segundo
(`max_fps` na secção `ui` do ficheiro de configuração), e os cliques e as
teclas são respondidos de imediato. Num computador antigo ou ocupado,
baixe-o (por exemplo para 20) para deixar mais do processador para o áudio; 0
desenha todas as imagens que o ecrã mostra. Os medidores medem o mesmo a
qualquer ritmo; só são desenhados com menos frequência.

## Não é possível largar ficheiros no Linux {#files-cannot-be-dropped-on-linux}

A janela só aceita ficheiros arrastados de um gestor de ficheiros quando corre
em X11. Por predefinição corre, também num ambiente de trabalho Wayland
(através do XWayland, que todos os ambientes têm). Se `ui.window_system` em
`config.json` estiver definido como `"Wayland"`, a janela é uma janela
Wayland nativa, que pode ficar mais nítida com escala de ecrã fracionária,
mas não é possível largar ficheiros nela: use **+ Adicionar**. Volte a
pô-lo em `"Auto"` (a predefinição) ou `"X11"` e reinicie o Fauste Player.

## Um controlador MIDI não responde {#a-midi-controller-does-not-respond}

Se **Definições → MIDI** mostra o controlador como ligado mas os seus botões
e faders não fazem nada, prima aí **Voltar a ligar as portas**: o programa
fecha as portas MIDI e abre-as de novo, como se desligasse e voltasse a ligar
o controlador. Se isso não ajudar, desligue o controlador e volte a ligá-lo.

De qualquer forma, o registo diz o que aconteceu. Lista as portas MIDI sempre
que mudam, cada porta aberta e fechada (com o motivo), uma porta que não
pode ser aberta (uma vez, até abrir ou desaparecer), a primeira mensagem
recebida em cada ligação, a primeira mensagem ignorada em cada ligação com o
motivo (sem associação, um fader que ainda não alcançou o volume, uma ação
que não pode atuar agora) e a primeira mensagem de LED que falhou. Um
controlador ligado que nunca regista uma primeira mensagem não está a enviar
nada que o programa receba. Para cada mensagem ignorada e cada nova tentativa
falhada, arranque o programa com `RUST_LOG=info,fp_control=debug`.
## «A interface teve um erro» {#the-interface-hit-an-error}

Foi detetado um erro de desenho. O áudio não é afetado. Prima **Reiniciar a
interface**. Comunique-o, por favor, com os registos.

## Registos e relatórios de falhas {#logs-and-crash-reports}

Consulte [Dados e cópias de segurança](data-and-backups.md) para a pasta de
registos. Há um ficheiro de registo por dia. As linhas idênticas que se
repetem umas a seguir às outras são escritas uma só vez, seguidas de uma linha
como `previous message repeated 12 times` (durante uma longa sequência de
repetições essa linha é escrita a cada minuto). As linhas que diferem, por
exemplo por levarem um contador, não são reduzidas. As mensagens rotineiras de
ligação do cliente PulseAudio são omitidas por predefinição. Defina
`RUST_LOG=debug` no ambiente para mais detalhe (substitui o filtro
predefinido, `info,pulseaudio=warn`).

O registo é limitado: o ficheiro de um dia deixa de crescer aos 10 MiB (a sua
última linha di-lo; o resto desse dia não é registado, o que é o que uma
inundação de linhas diferentes provoca) e os últimos 14 ficheiros são
mantidos. Altere-os no ficheiro de configuração com `limits.max_log_file_mb`
(1 a 1024), `limits.max_log_files` (2 a 365) e
`limits.log_repeat_summary_secs` (1 a 3600, predefinição 60, com que
frequência uma longa sequência de repetições é comunicada); aplicam-se no
arranque seguinte. Os relatórios de falhas são guardados como
`crash-<hora>.txt` na mesma pasta e são limitados em separado
(`limits.max_crash_reports` por execução). Anexe ambos os ficheiros ao
comunicar um erro.

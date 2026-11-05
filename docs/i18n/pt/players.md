# Leitores

Cada coluna é um leitor. Os leitores são independentes: cada um tem os seus
próprios separadores de listas, transporte, volume e saídas.

![O leitor 1 no ar: o seu cabeçalho com o ganho de loudness, capa, título, faixa seguinte, transporte, contagem decrescente, medidor, fader e forma de onda](../../images/guide/player.png)

## Cabeçalho {#header}

| Elemento | Significado |
|---|---|
| `P1` … `Pn` | Número do leitor (a tecla numérica que o faz tocar) |
| Botões **Ocultar** e **Maximizar** / **Restaurar** | Os dois botões pequenos na extremidade direita do cabeçalho: ocultam o leitor, ou maximizam-no por um momento (**Restaurar** enquanto está maximizado). Ficam esbatidos enquanto a disposição está bloqueada, o **Ocultar** no último painel visível e o **Maximizar** com um só painel visível (ver [Disposição](layout.md#maximize-a-panel)) |
| Ponto e etiqueta de estado | **No ar** (vermelho), **Em pausa** (âmbar), **Parado** (cinzento) |
| Colunas estreitas | Quando a coluna é demasiado estreita para todos os indicadores, o cabeçalho deixa cair primeiro os menos importantes (o texto de estado, **Outras silenciadas**, o aviso de entrada, **Stop no fim**, **Mistura** / **Fade**, o ponto de estado), para que nada se sobreponha. O número do leitor, **CUE**, o modo, **BP**, o indicador de ganho e os botões **Ocultar** e **Maximizar** ficam sempre. Alargue a coluna para os ver todos |
| Indicador **Mistura** / **Fade** | Está a decorrer uma mistura com a faixa seguinte, ou um stop com fade. A **Mistura** nunca se acende num leitor cuja saída Main é bit-perfect: não faz crossfade (ver [O que custa](bit-perfect.md#what-it-costs)) |
| Indicador **Stop no fim** | O leitor para quando a faixa atual termina |
| Indicador **Repetir** / **Parar após a faixa** | A faixa atual repete-se, ou para o leitor quando termina, por causa da sua própria marca no menu da lista. Passe o rato por cima para ver a frase completa. O botão **Stop no fim** do próprio leitor prevalece: enquanto está ativo, só aparece o seu indicador |
| Indicador de ganho | Com a [normalização do loudness](loudness.md#the-gain-in-the-player-header) ativada, o ganho com que toca a faixa no ar (**+4.0 dB**), ou um **0.0 dB** âmbar quando a faixa toca sem alterações (ainda sem medição, uma saída bit-perfect, ou uma faixa excluída da normalização). Passe o rato por cima para ver o motivo. Mostra-se enquanto o leitor toca ou está em pausa |
| **BP** / **DSD** | O indicador só aparece num leitor cujo dispositivo Main funciona em bit-perfect; o **BP** acende-se enquanto a faixa atual chega ao seu dispositivo Main sem alterações. O **DSD** substitui-o enquanto uma faixa DSD sai como DSD, sem alterações (ver [Saída bit-perfect](bit-perfect.md)). **Outras silenciadas** aparece ao lado quando esse fluxo DSD mantém as outras fontes fora da saída |
| **SINGLE** \| **CONT** | Modo de reprodução (ver abaixo): um controlo único, a metade acesa é o modo ativo |
| **CUE** | Pré-escutar a faixa seguinte na saída CUE |

## Linha de informação {#info-row}

- **Capa** da faixa no ar, ou um marcador de posição em forma de disco de
  vinil.
- **Título e artista** da faixa no ar. Um leitor parado mostra a faixa que o
  Play vai iniciar (a sua seguinte), com a sua capa, a sua duração e a sua
  forma de onda, pronta no cue-in, ou onde clicou na forma de onda.
- Uma faixa tocada ou carregada antes de a sua análise terminar já tem a sua
  duração quando o cabeçalho do ficheiro a indica: a contagem decrescente,
  `decorrido / total` e o clique para saltar funcionam de imediato, sobre
  uma linha plana até a forma de onda estar pronta. Quando o cabeçalho não a
  guarda (ficheiros AAC em bruto, ficheiros MP3 sem frame de duração,
  Matroska e WebM), o total mostra «—» e a forma de onda não pode ser
  clicada até a análise terminar.
- **Medidor estéreo** (na coluna à direita do leitor, junto ao fader; ocupa
  a linha de informação e o transporte): o nível que o leitor emite, depois
  do seu volume.
  - Uma barra contínua por canal, na escala da norma do tipo de medidor (o
    medidor de pico digital por predefinição: de −60 a 0 dBFS, com os 20 dB
    superiores a ocupar metade da altura). A escala é uma régua de cada lado
    das barras, com etiquetas nas unidades próprias do medidor em ambos os
    lados. O seu topo e a sua base (no medidor digital, a base da escala)
    estão sempre marcados; cada etiqueta tem um traço em cada régua, e
    traços mais curtos entre as etiquetas funcionam como os de uma régua de
    medição: espaçados de forma uniforme em valores redondos, a cada 1 dB no
    medidor EBU e a cada 5 dB abaixo de −20 no digital quando o medidor é
    suficientemente alto, e a cada 2, 2,5, 5 ou 10 dB (ou nenhum) quando é
    demasiado baixo para eles. No medidor digital (e no personalizado), um
    medidor alto etiqueta mais valores: a cada 1 dB de −20 a 0 e a cada 5 dB
    entre as marcas de 10 dB abaixo de −20 (−45, −55), sempre espaçados de
    forma uniforme e só onde cabem sem as etiquetas se tocarem; os outros
    tipos de medidor mantêm as etiquetas que a sua norma indica. O nível de
    alinhamento (−18 dBFS no medidor digital) é um traço branco mais grosso
    em ambas as réguas. Nada é desenhado sobre as barras nem entre elas,
    por isso o que vê nas barras é apenas o nível, a retenção do pico e as
    cores.
  - A barra é verde, amarela a partir do nível de aviso (−9 dBFS) e
    vermelha a partir do nível de perigo (−3 dBFS). Os outros tipos de
    medidor ficam vermelhos onde a sua escala o faz (a partir de 0 VU, a
    partir do máximo permitido num PPM).
  - Os medidores K-System mostram duas secções: a barra sólida é o nível
    médio (RMS) e a parte mais esbatida acima dela chega ao pico. As suas
    cores são as do K-System: verde abaixo de 0, âmbar de 0 a +4, vermelho
    acima.
  - Os medidores de pico digital, K-System e personalizado mantêm o nível
    mais alto aceso durante um momento como uma linha (a retenção do pico; a
    sua duração é **Retenção do pico** em
    [Definições → Medidores](settings.md#meters), e 0 desativa-a). Os
    medidores PPM EBU, PPM DIN e VU não têm retenção.
  - O número de cima é o nível mais alto desde que a faixa começou, em
    dBFS, vermelho na zona de perigo. Mantém-se após um stop e recomeça
    quando uma faixa toca (a seguinte ou a mesma de novo), ou quando clica
    nele.
  - O número de baixo é o loudness em LUFS (EBU R128), verde a ±1 LU do
    objetivo (−23 LUFS).
  - O tipo de medidor e cada nível podem ser alterados em
    [Definições → Medidores](settings.md#meters).
  - **Leituras acima de 0 dBFS.** O medidor mostra o que o leitor emite, e
    isso pode exceder a escala completa. A barra para no topo da escala,
    por isso o mesmo vermelho mostra 0 dBFS e qualquer valor acima; só o
    número de cima diz quanto, com o seu sinal (por exemplo, `+3.5`).
    - Um ficheiro pode trazer níveis acima da escala completa (um ficheiro
      float, ou um ficheiro com perdas cujos picos descodificados a
      excedem).
    - Converter a frequência de amostragem pode criar picos entre as
      amostras: um sinal que toca 0 dBFS lê cerca de +3 dBFS após 44,1 → 48
      kHz. A opção de true peak também lê esses picos.
    - O medidor lê cada leitor isoladamente, não a soma no dispositivo: dois
      leitores numa mesma saída podem somar acima da escala completa sem que
      nenhum dos medidores o mostre.
    - Nada no leitor acrescenta ganho acima de 100 %. Um dispositivo de
      inteiros corta na escala completa; um dispositivo float recebe o nível
      tal como está e o sistema de som ou o controlador corta-o.
- **Fader de volume** (à direita do medidor, com a mesma altura). Segue uma
  lei de mesa de mistura: é fino perto do topo e mais inclinado para a base,
  por isso baixá-lo um pouco baixa um pouco o nível (90 % do curso são cerca
  de −2,5 dB, 50 % cerca de −15,9 dB, 10 % cerca de −48 dB). O topo é 0 dB e
  a base é silêncio. Marcas ao longo do fader assinalam 0, −2, −4, −6, −8,
  −10, −12, −15, −20, −25, −30, −40, −50 e −∞ dB, com o nível escrito à sua
  direita.
  - Arraste-o: o fader move-se tanto quanto o ponteiro, a partir de onde
    está, por isso agarrá-lo nunca o faz saltar; um clique sem mover não
    altera nada. Puxar para lá do topo ou da base e voltar deixa o botão na
    ponta até o ponteiro regressar ao ponto onde lá chegou, para que o botão
    nunca fique longe do ponteiro.
  - Use a roda do rato: um passo por clique da roda (cerca de 0,6 dB perto
    de 0 dB).
  - Mantenha **Ctrl** ou **Shift** premido ao arrastar ou usar a roda para o
    mover dez vezes mais devagar.
  - Faça duplo clique no botão para voltar a 0 dB (um duplo clique noutro
    sítio do fader não faz nada).

  O nível em dB (uma casa decimal, ou −∞ para silêncio) está escrito por
  baixo do fader, na mesma linha que a linha de loudness do medidor, e a
  dica também o mostra. Uma sessão guardada antes desta lei mantém o nível de
  cada leitor; só muda a posição do fader.
  Num leitor cuja saída Main é bit-perfect, o fader fica bloqueado a 100 % e
  cinzento, lê um 0.0 esbatido, e a sua dica diz porquê (ver
  [O que custa](bit-perfect.md#what-it-costs)).
- **Título, artista** e a linha da **seguinte**, com um quadrado verde.
  Enquanto o CUE está ativo, a posição da pré-escuta aparece a azul.

## Transporte {#transport}

| Botão | Ação |
|---|---|
| **Play / Seguinte** (grande) | Parado: iniciar a faixa seguinte. No ar: fade para a faixa seguinte (o tempo de fade define-se em [Definições](settings.md)); enquanto faz o fade de um crossfade iniciado pelo Seguinte ou por um segue, uma faixa verde escoa-se pelo botão durante a duração do fade, com a margem a mover-se da esquerda para a direita como a de um cartucho (um crossfade iniciado pelo Anterior escoa-se no botão Anterior). Em pausa: retomar. Com a faixa no ar como seguinte, o Play reinicia essa faixa com o fade habitual. Numa saída bit-perfect muda de imediato, sem fade, e a sua dica di-lo (ver [O que custa](bit-perfect.md#what-it-costs)). |
| **Stop** | Parar de imediato (com uma curta rampa anti-estalido) |
| **Stop com fade** | Fade out e parar. Enquanto um stop com fade decorre, uma faixa escoa-se pelo botão durante a duração do fade, com a margem a mover-se da esquerda para a direita |
| **Pausa** | Pausar ou retomar; pisca a âmbar enquanto está em pausa |
| **Stop no fim** (um triângulo de play seguido de um quadrado) | Parar quando a faixa atual termina, uma só vez. No modo SINGLE só está disponível enquanto a faixa atual se repete: termina a repetição quando a passagem que está a tocar termina. Para parar após uma faixa sempre que toca, ou para repetir uma faixa, use o seu menu na lista (ver [Listas](playlists.md)) |
| **Anterior** (uma barra e dois triângulos) | No ar: fade de volta para a faixa que este leitor tocou antes, como faz o Seguinte. Prima de novo para continuar a recuar. A faixa que deixou passa a ser a seguinte. Numa saída bit-perfect muda de imediato, sem fade (ver [O que custa](bit-perfect.md#what-it-costs)). |
| **Reiniciar** (uma barra e um triângulo) | Voltar ao início da faixa atual (o seu cue-in). Um leitor em pausa continua em pausa. |

Pelo teclado, `Up` e `Down` movem a seleção pela lista, `Enter` define a faixa
selecionada como seguinte, e um segundo `Enter` nela dentro do tempo de um
duplo clique toca-a. Ver [Teclado](keyboard.md).

Os botões que não podem atuar neste momento ficam esbatidos: Stop e
Reiniciar sem nada carregado, Pausa e Stop com fade quando parado, Anterior
sem faixa anterior ou durante um fade. Um leitor recorda as últimas 50
faixas que tocou (`players.history_len` em `config.json`, de 0 a 1000).

## Modos {#modes}

- **CONT (contínuo):** no ponto MIX, o leitor inicia a faixa seguinte e
  sobrepõe o fim da atual. Ver [Marcadores e mistura](markers-and-mixing.md).
- **SINGLE:** cada faixa para no seu fim. O *Stop no fim* não está
  disponível neste modo, porque cada faixa já para, exceto enquanto a faixa
  atual se repete: então termina a repetição quando a passagem que está a
  tocar termina.

## Contagem decrescente {#countdown}

O número grande é o tempo que falta até ao fim da faixa (o seu cue-out), com
décimas. O tempo decorrido e o total estão na linha por baixo da forma de
onda, à direita. Durante os últimos segundos antes do fim (10 por
predefinição, definidos nas Definições) a contagem decrescente pisca a
vermelho.

## Forma de onda {#waveform}

- A parte já tocada é desenhada na cor da forma de onda; o resto é mais
  esbatido.
  Com uma cor de forma de onda violeta, a linha MIX violeta destaca-se menos;
  escolha outra cor de forma de onda se precisar de as distinguir.
- O contorno mostra os picos, ténues; o corpo sólido no seu interior é o
  nível médio (RMS). Numa faixa alta, os picos preenchem a altura, e o corpo
  continua a mostrar onde a faixa é mais baixa ou mais alta.
- Uma área sombreada a azul no início marca a **intro**, e um indicador
  conta-a em decrescente. A intro só aparece quando foi definida.
- Uma área sombreada a laranja no fim marca o **outro**, com a sua própria
  contagem decrescente.
- Uma linha tracejada violeta (a cor do indicador **Mistura**, para não se
  confundir com o outro laranja) com uma etiqueta **MIX** marca onde a faixa
  seguinte começa no modo contínuo. Fica esbatida no modo single, e num leitor
  cuja saída Main é bit-perfect, que não mistura (ver
  [O que custa](bit-perfect.md#what-it-costs)).
- O ficheiro inteiro é desenhado. O início e o fim silenciosos que a
  reprodução salta (antes do cue-in e depois do cue-out) são desenhados mais
  escuros, com uma linha fina onde a reprodução começa e acaba. Com **Usar
  cue-in e cue-out** desativado (Definições → Leitores) nada é mais escuro e
  as duas linhas ficam esbatidas: a reprodução vai do início ao fim do
  ficheiro, e o cue-in e o cue-out neste guia significam essas duas
  extremidades.
- Passe o rato para ver o tempo sob o ponteiro. **Clique para saltar** para
  aí. Num leitor parado, um clique escolhe onde o **Play** inicia a faixa
  seguinte: a posição de reprodução e a contagem decrescente movem-se para aí
  e nada toca até premir Play. Escolher outra faixa seguinte, movê-la ou
  removê-la, ou o Stop, regressa ao cue-in; o mesmo acontece com qualquer
  outra forma de iniciar uma faixa, e o Reiniciar, o Anterior e o avanço
  automático usam sempre o cue-in. Um clique antes do cue-in (no início mais
  escuro) escolhe o cue-in. Um clique no cue-out ou depois dele (na cauda
  mais escura) cancela uma escolha anterior: o Play começa no cue-in. Um
  clique é premir e soltar sem mover o ponteiro mais do que alguns píxeis.
- **Premir e arrastar** move a vista ampliada ao longo da faixa, como se a
  agarrasse. Um arrastamento nunca salta, e sem zoom não faz nada. Alt +
  arrastar continua a editar marcadores.
- **Roda do rato** sobre a forma de onda: aproxima e afasta em torno do
  ponteiro, até ao maior detalhe que a análise tem. **Shift+roda** (ou uma
  roda lateral) move ao longo da faixa. Com zoom, a vista segue a posição de
  reprodução, exceto durante 10 segundos depois de a ampliar ou mover
  (`ui.follow_current_grace_secs`; 0 desativa o seguimento). **Vista
  completa**, no canto superior direito, afastar o zoom por completo ou uma
  faixa nova mostram de novo a faixa inteira.

## CUE (pré-escuta) {#cue-pre-listen}

Premir **CUE** (ou **Pré-escutar no CUE** no menu de uma faixa) toca a faixa
na saída CUE do leitor, por exemplo uns auscultadores, e abre uma pequena
**janela de CUE** para esse leitor. Podem estar
abertas várias janelas, uma por leitor. Consulte as
[Definições](settings.md) para escolher o dispositivo de CUE. Um leitor
precisa de uma saída Cue: sem ela, **CUE** e **Pré-escutar no CUE** ficam
esbatidos, e passar o rato por cima explica porquê.

Com uma só placa de som, a saída Cue pode ser a própria saída do ar (ou
partilhá-la com outro leitor ou com a cartucheira). Aí, uma pré-escuta ouve-se
no ar, por isso:

- enquanto nada toca nessa saída, o CUE começa de imediato;
- enquanto algo toca aí (um leitor a tocar ou em pausa, um cartucho, outra
  pré-escuta, a pré-escuta da biblioteca), o CUE pergunta primeiro:
  **Pré-escutar mesmo assim** ou **Cancelar** (Cancelar é a predefinição: Esc
  e Enter cancelam). A mesma pergunta vem do menu da faixa, da janela de CUE,
  da cartucheira e da janela da biblioteca;
- quando algo começa nessa saída enquanto toca aí uma pré-escuta que não
  confirmou, a pré-escuta pára e a barra de estado diz porquê. Uma
  pré-escuta que confirmou continua a tocar.

A dica do botão CUE diz quando a sua pré-escuta sai pela saída do ar.

![A janela de CUE do leitor 4, a pré-escutar a sua faixa seguinte](../../images/guide/cue-window.png)

A janela mostra:

- o título e o artista;
- a forma de onda do ficheiro inteiro com a posição do CUE. Funciona como a
  do leitor: clique para saltar para aí, zoom com a roda, arrastar para mover
  ao longo, **Vista completa**, as contagens decrescentes da intro e do
  outro, e os marcadores de intro, outro e MIX, que edita aqui como no leitor
  (ver [Marcadores e mistura](markers-and-mixing.md)). Um CUE toca o
  ficheiro inteiro, por isso nada é desenhado mais escuro, o cue-in e o
  cue-out são linhas esbatidas e o outro conta em decrescente até ao fim do
  ficheiro. O seu zoom é próprio: a forma de onda do leitor não se move.
  Enquanto o CUE toca, uma vista ampliada segue a sua posição como a do
  leitor; um CUE em pausa mantém a vista que definiu, para poder ampliar e
  colocar marcadores. Um CUE iniciado depois de a sua janela ter fechado, ou
  noutra faixa, mostra o ficheiro inteiro;
- o tempo decorrido e o tempo que falta até ao fim do ficheiro (um CUE toca
  ficheiros inteiros);
- **Pausar o CUE** / **Retomar o CUE**, **Parar o CUE** e **Definir como
  seguinte**. **Definir como seguinte** torna a faixa em CUE a seguinte do
  leitor e mantém o CUE a tocar. Fica esbatido quando a faixa já é a
  seguinte. Se a faixa em CUE for a que está no ar, toca mais uma vez quando
  a passagem atual terminar.

Enquanto o CUE está em pausa, o seu botão de pausa (que mostra **Retomar o
CUE**) pisca a âmbar, como o do próprio leitor.

Um salto num CUE em pausa mantém-no em pausa. O botão de fechar da janela,
ou **Parar o CUE**, para o CUE.

Enquanto um CUE decorre, definir uma seguinte (duplo clique) ou um único
clique numa linha leva o CUE para essa faixa, a partir do seu cue-in; se
estava em pausa, volta a tocar. Uma faixa cujo ficheiro está em falta ou é
ilegível deixa o CUE onde está.

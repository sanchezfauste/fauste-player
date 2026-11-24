# Automatização

A automatização mantém a estação no ar quando não há ninguém no estúdio. Esta
versão tem o **musicalizador**, um leitor que escolhe a música sozinho, a
partir das regras que define, e nunca pára por falta de lista, e a coluna
**Grelha**, que executa os blocos horários do dia, feitos a partir de modelos
nas Definições.
Os leitores que já usa (P1 … Pn) não são tocados: continuam a ser seus para
os programas que prepara e emite em direto.

## A coluna Grelha {#the-log-column}

A coluna **Grelha** reproduz os **blocos** do dia: grupos de elementos com uma
hora de início (um indicativo horário, spots, um programa). Mostre-a a partir
do mosaico **Grelha** da barra superior, como mostra um leitor.

![A coluna Grelha em funcionamento, ao lado da coluna Musicalizador: os blocos do dia por hora de início, cada um com o seu indicativo da estação (o relógio marca-o como de hora fixa), spots e a rotação que o musicalizador reproduz depois; a canção do musicalizador no ar e as seguintes](../../images/guide/automation-columns.png)

A sua lista parece uma playlist, agrupada por blocos. Cada bloco começa com
uma linha de cabeçalho: a sua hora de início, quantos elementos tem e quanto
duram, "alterado para este dia" quando o alterou hoje e, a âmbar, quanto se
estende para além do bloco seguinte. Depois vêm os seus elementos, com a
**Hora** a que cada um está previsto começar. A lista mostra hoje, o último
bloco de ontem e o primeiro de amanhã. O elemento no ar e o seguinte são
marcados como num leitor; os elementos que já foram emitidos aparecem
esbatidos, e um elemento que não foi emitido (**não emitido**) aparece
riscado. Um relógio marca um elemento de **hora fixa**, que só toca à sua
hora. Uma **faixa de categoria** mostra a faixa escolhida para ela assim que
passa a ser a seguinte (ou vem depois da seguinte no seu bloco), com a
categoria a seguir: é essa a faixa que toca. Se a categoria não tiver nenhuma
faixa que possa tocar (nenhuma lhe está atribuída, ou faltam os seus
ficheiros), a linha di-lo a âmbar, e o elemento não será emitido. Um elemento
cujo ficheiro se revela ilegível quando está prestes a tocar é saltado e
também marcado como não emitido. As faixas atribuem-se a uma categoria na
biblioteca. A linha do **musicalizador** diz até quando toca, e aparece no ar
enquanto o musicalizador toca para a Grelha. Um seguinte cuja hora já passou
mostra a hora a âmbar. Arraste a linha entre dois cabeçalhos de coluna para
mudar a largura das colunas, como numa playlist; as janelas de modelo e de
dia usam as mesmas larguras.

### Alterar a grelha de hoje {#changing-todays-log}

A Grelha altera-se como uma playlist, e a alteração é só para esse dia: o
modelo fica como está.

- Os botões no fundo da coluna: **+ Faixa…** abre a biblioteca, para arrastar
  faixas para a lista; **+ Faixa de categoria…** e
  **+ Musicalizador (rotação)…** perguntam qual e adicionam-na logo abaixo da
  linha que selecionou (clique numa linha para a selecionar; selecione um
  cabeçalho de bloco para adicionar no início desse bloco), ou, sem nada
  selecionado ou abaixo de uma linha que já não pode mudar, no fim do próximo
  bloco; **+ Bloco horário…** pede uma hora, com a primeira livre já
  preenchida. Quando a coluna é demasiado estreita para todos, os que não
  cabem ficam atrás de **⋯**.
- Arraste faixas da biblioteca, ou ficheiros de áudio do seu gestor de
  ficheiros, para uma linha para os pôr antes dela (ou depois, na metade
  inferior da linha, como numa playlist: é assim que vão para o fim de um
  bloco), e arraste linhas dentro do mesmo dia para as mover. A linha que
  adiciona fica selecionada e à vista, por isso a seguinte que adicionar vai
  para baixo dela.
- Clique com o botão direito numa linha para **Hora fixa** e **Remover da
  grelha de hoje**. Clique (ou clique com o botão direito) num cabeçalho de
  bloco para adicionar uma **Faixa…**, uma **Faixa de categoria…** ou um
  **Musicalizador (rotação)…** no fim desse bloco, ou para **Alterar hora…**,
  **Remover bloco** e, num bloco alterado, **Voltar ao modelo**.
- Enquanto arrasta sobre um cabeçalho de bloco, a linha por baixo mostra que
  larga no início desse bloco; para pôr linhas no fim do bloco de cima,
  largue-as na metade inferior da sua última linha.
- O que já tocou, o elemento no ar e os dias passados não podem ser
  alterados. Uma alteração ao bloco no ar tem efeito depois do elemento no
  ar.

A coluna Grelha tem o cabeçalho e os botões de um leitor, e funcionam como
num leitor:

- **Reproduzir** reproduz o elemento marcado como seguinte e põe a
  automatização a funcionar: cada bloco toca então à sua hora, e o
  musicalizador toca entre blocos. Quando o seguinte foi escolhido pelo
  relógio e o seu bloco começa mais tarde, a música toca até lá. Nada arranca
  sozinho: depois de cada arranque da aplicação, a automatização está parada
  até premir Reproduzir.
- **Pausa**, **Stop** e **Stop com fade** atuam sobre o que está no ar para a
  Grelha (o seu elemento, ou a canção do musicalizador entre blocos) e param
  a automatização. Reproduzir depois de Pausa continua de onde estava;
  enquanto está em pausa, o seguinte não se move, e ao retomar o bloco
  seguinte mantém a sua hora. Reproduzir depois de um Stop durante a música
  volta a pôr música, e o bloco seguinte começa à sua hora.
- **Stop no fim** deixa acabar o que está no ar, e depois a automatização
  pára. Se a música estiver a tocar e a hora de um bloco chegar primeiro, o
  bloco começa a horas e a automatização pára depois do seu primeiro
  elemento.
- **Faixa anterior** não faz nada na Grelha.
- **SINGLE** torna a Grelha manual: Reproduzir reproduz o elemento seguinte, e
  pára depois dele; nenhum bloco começa sozinho.

Enquanto o musicalizador toca entre blocos, o cabeçalho da Grelha diz
**Musicalizador · *rotação*** (a rotação que reproduz) e os seus botões
continuam a funcionar; a canção, o seu medidor e a sua forma de onda estão só
na coluna Musicalizador.

Quando a automatização pára depois de um elemento, o seguinte fica no
elemento abaixo do que tocou, e Reproduzir reprodu-lo, seja qual for a hora:
escolhe o utilizador. Quando pára durante a música, Reproduzir volta a pôr
música até ao bloco seguinte, a menos que tenha escolhido outro seguinte.
Para mudar o que toca a seguir, faça duplo clique num elemento, ou clique nele
com o botão direito e escolha **Tocar a seguir**. **Voltar ao relógio**, no
mesmo menu, põe como seguinte o que diz a hora.

## Modelos e dias {#templates-and-days}

A Grelha reproduz cada dia a partir de um **modelo**: um dia de blocos. Uma
instalação nova tem um, "Base", para todos os dias. Os modelos fazem-se em
Definições → **Automatização**, em **Modelos e dias** (não no Android); a
coluna Grelha só altera o dia de hoje, como acima.

![Definições, Automatização, Modelos e dias: os modelos Base e Weekend com Editar…, Novo modelo, Duplicar, Eliminar e o nome; a semana tipo, Base de segunda a sexta e Weekend ao sábado e domingo; o mês com hoje escolhido, as datas com alterações a âmbar, o modelo da data e Editar dia…](../../images/guide/settings-plans.png)

- **Modelos.** Escolha um e prima **Editar…** (ou faça duplo clique nele)
  para o abrir; **Novo modelo**, **Duplicar** e **Eliminar** estão ao lado, e
  o seu nome por baixo.
- **Semana tipo.** Que modelo usa cada dia da semana.
- **Dias.** Escolha uma data no calendário (as datas com alterações estão a
  âmbar). Uma data pode usar um modelo diferente do do seu dia da semana (um
  feriado a uma quinta-feira pode usar o do fim de semana). **Editar dia…**
  abre essa data, para alterar os seus blocos só para essa data; as datas
  passadas não podem ser alteradas.

Um modelo ou uma data abre-se na sua própria janela, que fecha as
Definições. É a mesma lista que a coluna Grelha, e altera-se da mesma forma:
os botões no fundo (Faixa…, Faixa de categoria…, Musicalizador (rotação)…,
Bloco horário…, que adicionam abaixo da linha que seleciona), faixas
arrastadas da biblioteca (ou ficheiros do seu gestor de ficheiros) para uma
linha, linhas arrastadas para as mover, o menu do botão direito das linhas e
o menu dos cabeçalhos de bloco. Nunca toca, e pode ficar aberta ao lado da
biblioteca. A janela de hoje mostra o que tocou e o que toca, como a Grelha;
a janela de um dia fecha-se à meia-noite, quando esse dia já não pode mudar.

![Um modelo aberto na sua janela: os blocos do modelo Base por hora de início, cada um com o seu indicativo da estação, os seus spots e o musicalizador depois, e Adicionar no fundo](../../images/guide/template-popup.png)

Uma **faixa de categoria** num modelo reproduz uma faixa dessa categoria, uma
diferente de cada vez. O elemento **Musicalizador** escolhe a rotação que o
musicalizador reproduz depois do bloco (a rotação predefinida quando o bloco
não tem nenhuma). Quando os elementos de um bloco duram mais do que o tempo
até ao bloco seguinte, o seu cabeçalho diz quanto, a âmbar; mesmo assim pode
deixá-lo assim.

Uma alteração a uma data fica marcada como "alterado para este dia" no seu
bloco, e **Voltar ao modelo**, no menu do cabeçalho do bloco, desfá-la. As
alterações posteriores ao modelo continuam a chegar aos blocos que não
alterou nessa data. Uma alteração a um modelo feita enquanto um dos seus
blocos está no ar deixa esse bloco como está para hoje (mostra "alterado para
este dia"); a alteração chega-lhe a partir do dia seguinte.

## O musicalizador {#the-music-fill}

O musicalizador é a coluna **Musicalizador**. Está oculta até a mostrar:
clique no mosaico **Musicalizador** à esquerda da barra superior. Fica à
esquerda de P1.

Tem o aspeto e o funcionamento de um leitor: o relógio grande, a forma de
onda, os botões de transporte, o fader e a lista de faixas. A diferença está
na lista:

- **enche-se sozinha** a partir de uma **rotação** (abaixo), sempre com cerca
  de meia hora de avanço (a *margem à frente*, em Definições →
  Automatização);
- **apara-se sozinha**: mantém as últimas 50 faixas que reproduziu e remove as
  mais antigas, para poder funcionar durante semanas;
- antes de um bloco da grelha pode **trocar canções que escolheu ela própria**
  por outras cuja duração caiba no tempo que falta (ver abaixo); as canções
  que adicionou nunca são tocadas;
- ainda pode **alterá-la à mão**: arrastar faixas da biblioteca, reordenar
  linhas, remover uma canção que não quer. Uma linha que adiciona é marcada
  com uma mão e nunca é substituída; uma linha que remove não conta como
  reproduzida.

As suas saídas definem-se em Definições → Saídas de áudio, na linha
**Musicalizador**, como para qualquer leitor. Não tem número, nem atalhos de
teclado nem associações MIDI, e o controlo remoto não a vê.

Prima **Reproduzir** para a pôr a funcionar e **Stop no fim** para deixar
acabar a canção que está no ar e parar. Como todos os leitores, nunca arranca
sozinha, nem sequer depois de um reinício. Enquanto toca para a Grelha entre
blocos, pará-la ou pô-la em pausa também pára a automatização, e o seu Stop
no fim pára a automatização quando a canção acaba; se a hora de um bloco
chegar primeiro, o bloco toca e a marca é retirada. Depois de a Grelha ter
sido posta em pausa durante a música, Reproduzir na coluna Musicalizador
reproduz a canção por conta própria: o Reproduzir da Grelha já não a retoma,
e o seu Stop já não a pára.

Por cima da sua lista, onde um leitor tem os separadores das listas, a coluna
Musicalizador mostra a **rotação** que reproduz. Escolha outra aí: as canções
que tinha em fila são substituídas, as que adicionou ficam. Quando a rotação
não encontra nada para reproduzir (as suas categorias estão vazias, ou todos
os ficheiros estão em falta), a barra di-lo.

## Rotações {#rotations}

Uma **rotação** é uma lista curta de **passos** que o musicalizador reproduz
à vez, e depois de novo a partir do primeiro. Cada passo indica de onde vêm as
suas faixas (uma categoria da biblioteca com as suas subcategorias, ou todas
as faixas de um tipo) e quantas reproduz de cada vez, como um intervalo:

| Passo | Reproduz |
|---|---|
| Current hits | 2–3 |
| Station IDs | 1 |
| Gold | 0–1 |

Com um intervalo, um indicativo da estação vem depois de duas canções numa
vez e depois de três na seguinte. Um intervalo a partir de **0** deixa um
passo ficar de fora em algumas voltas: com Gold a 0–1, uma canção gold toca
depois do indicativo da estação umas vezes e outras não. O máximo é sempre
pelo menos 1. Uma rotação é a **predefinida**: uma instalação nova tem uma,
**Música**, que reproduz toda a sua música.

Um passo cuja categoria não tem nenhuma faixa que possa tocar (uma categoria
vazia, ou ficheiros em falta) toca na sua vez uma canção da rotação
predefinida (qualquer música, se o passo for da própria rotação
predefinida), para que os outros passos mantenham o seu lugar: um indicativo
da estação continua a vir entre canções.

As rotações criam-se em Definições → Automatização:

![Definições, Automatização: as rotações Music (a predefinida) e Hits, a Music selecionada com o seu único passo sobre toda a música, as suas contagens No mínimo e No máximo, e o início de Modelos e dias](../../images/guide/settings-automation.png)

## Como escolhe {#how-it-chooses}

Dentro de um passo, o musicalizador prefere as faixas que estão há mais tempo
sem ir para o ar, com um pouco de acaso entre elas para que a ordem nunca se
torne um ciclo fixo. Também mantém afastados:

- **a mesma faixa:** pode tocar de novo quando passarem 3 horas ou quando
  metade das outras faixas do seu passo tiverem sido emitidas desde então, o
  que ocorrer primeiro;
- **o mesmo artista:** não nos 30 minutos seguintes.

Conta tudo o que foi para o ar, do musicalizador, de um leitor ou da
cartucheira; a pré-escuta não conta. Quando uma categoria pequena não
consegue manter as regras, o musicalizador cede em vez de ficar em silêncio:
primeiro no artista, depois na faixa. Definições → Automatização tem todos os
valores, e é onde se criam, renomeiam e alteram as rotações (ver
[Definições](settings.md#automation)).

Antes de um bloco da grelha, o musicalizador também escolhe as suas últimas
canções pela duração, para que a música acabe de forma limpa: a canção que
desvanece quando o bloco começa ainda está a tocar à hora do bloco e perde no
máximo 30 segundos depois dela, e não começa nenhuma canção que tocasse menos
de 45 segundos. Também mantém as regras de artista e de faixa sempre que
pode. Quando nenhuma canção da rotação consegue cumprir as duas coisas, toca
a que menos perde, pelo que por vezes se podem ouvir uns segundos de uma
canção. Uma pausa da música desloca as canções que vêm depois, e o
musicalizador volta a escolhê-las. O musicalizador também evita as canções, e
os artistas, que um bloco reproduz pouco depois. Por agora, os dois valores
só podem ser alterados no ficheiro de configuração
(`automation.max_fade_cut_secs` e `automation.min_song_play_secs`).

Um **Indicativo horário** é um tipo de faixa próprio (Biblioteca → Tipo ▸
Indicativo horário), para a identificação da estação à hora certa.

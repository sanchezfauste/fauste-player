# Automatização

A automatização mantém a estação no ar quando não há ninguém no estúdio. Esta
versão tem o **musicalizador**, um leitor que escolhe a música sozinho, a
partir das regras que define, e nunca pára por falta de lista, e a coluna
**Grelha**, que executa os blocos horários do dia, com o editor onde os cria.
Os leitores que já usa (P1 … Pn) não são tocados: continuam a ser seus para
os programas que prepara e emite em direto.

## A coluna Grelha {#the-schedule-column}

A coluna **Grelha** reproduz os **blocos** do dia: grupos de elementos com uma
hora de início (um indicativo horário, spots, um programa). Mostre-a a partir
do mosaico **Grelha** da barra superior, como mostra um leitor.

![A coluna Grelha em funcionamento, ao lado da coluna Música: os blocos do dia por hora de início, cada um com o seu indicativo da estação (o relógio marca-o como pontual), spots e a rotação que o musicalizador reproduz depois; a canção do musicalizador no ar e as seguintes](../../images/guide/automation-columns.png)

A sua lista é a grelha, agrupada por blocos: a hora de início de cada bloco e
depois os seus elementos. Mostra hoje, o último bloco de ontem e o primeiro de
amanhã. Um bloco alterado para um dia di-lo (*alterado para este dia*). O
elemento no ar e o seguinte são marcados como num leitor; os elementos que já
foram emitidos aparecem esbatidos, e um elemento que não foi emitido (**Não
emitido**) aparece riscado. Um relógio marca um elemento **pontual**, que só
toca à sua hora.

Por cima da lista:

- **Iniciar** põe a grelha a correr: cada bloco toca à sua hora, e o
  musicalizador toca entre blocos. Nada corre sozinho: depois de cada
  arranque da aplicação, a automatização está parada até premir Iniciar.
- **Parar após a atual** deixa terminar o que está no ar, e depois nada mais
  toca sozinho.
- **Em curso** ou **Parada** diz qual dos dois.

Para mudar o que toca a seguir, faça duplo clique num elemento, ou clique com
o botão direito e escolha **Tocar a seguir**; segue-se à canção no ar.
**Voltar ao relógio**, no mesmo menu, deixa a grelha voltar a escolher pela
hora.

## Editar a grelha {#editing-the-schedule}

**Editar grelha**, na coluna Grelha, abre o editor. Tem três separadores, e
**Rotações…** leva-o a Definições → Automatização, onde as rotações são
criadas.

![O editor da grelha, separador Modelos: os modelos Base e Weekend à esquerda; os blocos de Base à direita, cada um com o seu início, os seus elementos, Adicionar de categoria e a rotação para a música depois dele](../../images/guide/schedule-templates.png)

- **Modelos.** Um modelo é um dia de blocos; uma instalação nova tem um,
  "Base", para todos os dias. Crie, duplique, renomeie ou elimine modelos à
  esquerda; os blocos do escolhido estão à direita. Escreva o início de um
  bloco como `HH:MM:SS` (uma hora que não existe fica no campo com uma nota,
  e nada muda). Arraste faixas da biblioteca para um bloco para as adicionar;
  **Adicionar de categoria** adiciona um elemento que toca uma faixa dessa
  categoria, uma diferente de cada vez; **Música depois** escolhe a rotação
  que o musicalizador reproduz depois do bloco (a rotação predefinida quando
  nenhuma é escolhida). Cada elemento pode ser marcado como **Pontual**,
  movido ou removido. Quando os elementos de um bloco são mais longos do que o
  tempo até ao bloco seguinte, o bloco diz por quanto, a âmbar; pode mantê-lo
  assim na mesma.
- **Semana tipo.** Que modelo usa cada dia da semana.

  ![O separador Semana tipo: Base de segunda a sexta, Weekend ao sábado e ao domingo](../../images/guide/schedule-week.png)

- **Dias.** Escolha uma data no calendário (as datas com alterações aparecem a
  âmbar). Uma data pode usar outro modelo que não o do seu dia da semana (um
  feriado a uma quinta-feira pode usar o do fim de semana). Os seus blocos
  podem ser alterados só para essa data: a alteração fica marcada, e **Voltar
  ao modelo** desfá-la. **Neste dia não** remove um bloco só dessa data, e
  **Adicionar bloco neste dia** adiciona um. As alterações posteriores ao
  modelo continuam a chegar aos blocos que não alterou nessa data.

  ![O separador Dias: o calendário com hoje escolhido e as datas com alterações a âmbar; o primeiro bloco de hoje tem mais um spot, marcado "alterado para este dia", com Voltar ao modelo](../../images/guide/schedule-days.png)

## O musicalizador {#the-music-filler}

O musicalizador é a coluna **Música**. Está oculta até a mostrar: clique no
mosaico **Música** à esquerda da barra superior. Fica à esquerda de P1.

Tem o aspeto e o funcionamento de um leitor: o relógio grande, a forma de
onda, os botões de transporte, o fader e a lista de faixas. A diferença é a
lista:

- **preenche-se sozinha** a partir de uma **rotação** (abaixo), sempre cerca
  de meia hora à frente (*margem à frente*, em Definições → Automatização);
- **apara-se sozinha**: guarda as últimas 50 faixas que tocou e remove as mais
  antigas, para poder correr durante semanas;
- antes de um bloco da grelha pode **trocar as canções que escolheu
  sozinho** por outras cuja duração se ajusta ao tempo que resta (ver abaixo);
  as canções que adicionou nunca são tocadas;
- pode na mesma **alterá-la à mão**: arraste faixas da biblioteca, reordene
  linhas, remova uma canção que não queira. Uma linha que adiciona é marcada
  com uma mão e nunca é substituída; uma linha que remove não conta como
  tocada.

As suas saídas definem-se em Definições → Saídas de áudio, na linha
**Música**, como em qualquer leitor. Não tem número, nem atalhos de teclado
nem associações MIDI, e o controlo remoto não a vê.

Prima **Play** para a iniciar e **Parar após a atual** para deixar terminar a
canção no ar e parar. Como todos os leitores, nunca arranca sozinha, nem
depois de um reinício.

Por cima da sua lista, onde um leitor tem os separadores das listas, a coluna
Música mostra a **rotação** que reproduz. Escolha outra aí: as canções que
tinha em fila são substituídas, as que adicionou ficam. Quando a rotação não
encontra nada para reproduzir (as suas categorias estão vazias, ou todos os
ficheiros estão em falta), a barra di-lo.

## Rotações {#rotations}

Uma **rotação** é uma lista curta de **passos** que o musicalizador reproduz
à vez, e depois de novo desde o primeiro. Cada passo diz de onde vêm as suas
faixas (uma categoria da biblioteca com as suas subcategorias, ou todas as
faixas de um tipo) e quantas reproduz de cada vez, como um intervalo:

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

As rotações criam-se em Definições → Automatização:

![Definições, Automatização: as rotações Music (a predefinida) e Hits, a Music selecionada com o seu único passo sobre toda a música, as suas contagens No mínimo e No máximo, e as regras do musicalizador](../../images/guide/settings-automation.png)

## Como escolhe {#how-it-chooses}

Dentro de um passo, o musicalizador prefere as faixas que estão há mais tempo
sem ir para o ar, com um pouco de acaso entre elas para que a ordem nunca se
torne um ciclo fixo. Mantém também afastados:

- **a mesma faixa:** pode tocar de novo quando passarem 3 horas ou quando
  metade das outras faixas do seu passo tiverem sido emitidas desde então, o
  que ocorrer primeiro;
- **o mesmo artista:** não nos 30 minutos seguintes.

Conta tudo o que foi para o ar, do musicalizador, de um leitor ou da
cartucheira; a pré-escuta não conta. Quando uma categoria pequena não
consegue manter as regras, o musicalizador cede em vez de ficar em silêncio:
primeiro no artista, depois na faixa. Definições → Automatização tem todos os
valores, e é onde cria, renomeia e altera rotações (ver
[Definições](settings.md#automation)).

Antes de um bloco da grelha, o musicalizador escolhe também as suas últimas
canções pela duração, para que a música termine limpa: a canção que faz fade
out quando o bloco começa perde no máximo 30 segundos, e nenhuma canção
começa se fosse tocar menos de 45 segundos. Quando nenhuma canção da rotação
cumpre as duas condições, toca a que perde menos. Ambos os valores só podem
ser alterados, por agora, no ficheiro de configuração
(`automation.max_fade_cut_secs` e `automation.min_song_play_secs`).

Um **Indicativo horário** é um tipo de faixa próprio (Biblioteca → Tipo ▸
Indicativo horário), para a identificação da estação no início da hora.

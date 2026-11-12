# Disposição

Organize a janela principal para o seu turno: dê aos leitores e à cartucheira
os tamanhos que quiser, ponha a cartucheira por baixo dos leitores ou ao
lado deles, oculte os painéis que não usa e maximize um por um momento. A
disposição é guardada com a sessão e regressa no arranque seguinte; uma
instalação nova começa com todos os leitores com a mesma largura e a
cartucheira por baixo deles.

## Mostrar e ocultar painéis {#show-and-hide-panels}

Os mosaicos à esquerda da barra superior, **Grelha** e **Música** (a
[grelha](automation.md#the-schedule-column) e o
[musicalizador](automation.md#the-music-filler), ambos ocultos até os
mostrar), **P1** … **Pn** e **Cartucheira**, mostram que painéis estão no
ecrã: um mosaico aceso está visível. Clique num mosaico para ocultar ou
mostrar o seu painel. Com muitos leitores, os mosaicos deslocam-se para os
lados (rode a roda do rato sobre eles) e o botão da disposição continua à
vista. Os outros painéis ocupam o espaço que fica livre, e um leitor
mostrado de novo regressa à sua largura.

Ocultar apenas tira um painel do ecrã. Um leitor oculto continua a tocar, e
a sua tecla numérica, o MIDI, a API remota, a sua janela de CUE e a
automação continuam a controlá-lo. Um mosaico mostra o estado do seu painel,
quer o painel esteja visível ou oculto: um ponto vermelho enquanto um leitor
toca (ou a cartucheira enquanto um cartucho toca) e duas barras âmbar, como
um botão de Pausa, enquanto um leitor está em pausa.

Ocultar um leitor limpa a sua faixa selecionada, por isso **Delete** não
pode remover uma faixa que não vê. Os ficheiros largados na janela longe de
uma lista vão para o último leitor que usou, ou para o primeiro leitor
visível; com todos os leitores ocultos são recusados, e a barra de estado
di-lo.

O mosaico **Biblioteca**, a seguir a eles, abre e fecha a [janela da
biblioteca](library.md), que flutua sobre os painéis; não é um painel, por
isso a regra do último mosaico aceso abaixo não se lhe aplica. Não é
mostrado num telemóvel.

Pelo menos um painel fica visível: o último mosaico aceso não pode ser
clicado. Enquanto a cartucheira está oculta, o atalho que a recolhe ou
expande não faz nada; clique em **Cartucheira** para a mostrar.

## Redimensionar os leitores {#resize-the-players}

Arraste o espaço entre dois leitores para passar largura de um para o outro.
Os outros leitores não se movem, e nenhum leitor fica mais estreito do que os
seus controlos precisam. Faça duplo clique no espaço para dar de novo a todos
os leitores a mesma largura.

Quando a janela é demasiado estreita para todos os leitores visíveis no seu
mínimo, a fila de leitores desloca-se para os lados, como sempre.

## Mover e redimensionar a cartucheira {#move-and-resize-the-cartwall}

A cartucheira fica por baixo dos leitores, ou à esquerda ou à direita da
janela em toda a sua altura: escolha **Cartucheira em baixo**,
**Cartucheira à esquerda** ou **Cartucheira à direita** no [menu da
disposição](#the-layout-menu). Por baixo dos leitores tem a altura que as
suas linhas precisam, no máximo seis décimos da altura entre a barra superior
e a barra de estado; ao lado deles ocupa, no início, três décimos da largura.

A cartucheira nunca é mais estreita do que o seu cabeçalho (**Parar tudo**,
o seu título e um separador de página) nem do que as suas colunas de botões
no mínimo. Quando a janela é demasiado estreita para ela e para todos os
leitores visíveis no mínimo, a fila de leitores desloca-se para os lados.

Arraste o espaço entre a cartucheira e os leitores para a aumentar ou
diminuir: para cima ou para baixo quando está por baixo dos leitores, para os
lados quando está ao lado. Os leitores mantêm o espaço que os seus controlos
precisam. Faça duplo clique no espaço para voltar ao tamanho inicial: com a
altura que as suas linhas precisam por baixo dos leitores, três décimos da
largura ao lado deles. O tamanho é guardado com a disposição; mover a
cartucheira recomeça-a no seu tamanho inicial.

Depois de redimensionar a cartucheira, ou enquanto está ao lado dos leitores,
os seus botões crescem para a preencher; os nomes e os tempos mantêm o
tamanho, no meio de cada botão. Um nome quebra entre as suas palavras; uma
palavra demasiado longa para o seu botão termina em reticências (…) em vez de
se partir por várias linhas. Quando a cartucheira é mais pequena do que as
suas linhas e colunas precisam com os botões no mínimo, desloca-se.

Clique em **CARTUCHEIRA** para recolher a cartucheira. Por baixo dos
leitores só fica o seu cabeçalho; ao lado deles torna-se uma faixa estreita
com a altura da janela, com uma seta para a expandir de novo e, enquanto um
cartucho toca, um ponto vermelho. Expandida, recupera o seu tamanho. Enquanto
está recolhida, o espaço a seu lado não faz nada.

## Maximizar um painel {#maximize-a-panel}

O cabeçalho de cada leitor, e o da cartucheira, termina com dois botões
pequenos: **Ocultar** (um menos) e **Maximizar** (cantos a apontar para fora;
**Restaurar**, cantos a apontar para dentro, no painel maximizado). Passe o
rato por cima para ver o nome. Ficam esbatidos enquanto a disposição está
bloqueada, o **Ocultar** no último painel visível, e o **Maximizar** com um
só painel visível e numa cartucheira recolhida. As mesmas duas ações estão no
menu de um clique com o botão direito no fundo do cabeçalho (não num dos seus
botões):

- **Maximizar** dá a esse painel todo o espaço: os outros painéis encolhem
  para o seu mínimo. É só por um momento e não é guardado. **Restaurar** no
  mesmo menu termina-o, e também o terminam ocultar ou mostrar um painel,
  bloquear a disposição, mover a cartucheira, alterar o número de leitores e
  arrastar ou fazer duplo clique num espaço entre painéis; os tamanhos de
  antes regressam. Uma cartucheira recolhida não pode ser maximizada, e
  recolher uma cartucheira maximizada termina-o.
- **Ocultar** oculta o painel, como faz o seu mosaico na barra superior.

## O menu da disposição {#the-layout-menu}

O botão da disposição, à direita dos mosaicos, abre um menu com:

- **Cartucheira em baixo**, **Cartucheira à esquerda**, **Cartucheira à
  direita**: onde fica a cartucheira;
- **Bloquear a disposição**: enquanto está assinalado, nada altera a
  disposição por acidente: os mosaicos, as outras entradas deste menu, os
  espaços entre os painéis e o **Maximizar** e o **Ocultar** dos cabeçalhos
  não fazem nada. A reprodução, e recolher ou expandir a cartucheira,
  funcionam como de costume. O bloqueio é guardado com a disposição;
- **Leitores com a mesma largura**: todos os leitores com a mesma largura;
- **Repor a disposição**: volta à disposição de uma instalação nova.

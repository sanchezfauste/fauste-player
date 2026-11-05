# Biblioteca

A biblioteca é todas as faixas que o Fauste Player conhece: cada ficheiro que
adicionou a uma lista, a um cartucho ou à própria biblioteca é uma faixa, com
as suas etiquetas, marcadores e definições de loudness. A janela da
biblioteca lista-as, agrupa-as em *categorias*, encontra-as por texto e
envia-as para as listas e para a cartucheira. É uma janela no computador e
num tablet; um telemóvel mostra a biblioteca no seu separador **Biblioteca**
(ver [Num telemóvel](#on-a-phone)).

## Abrir a janela {#opening-the-window}

Clique no mosaico **Biblioteca** na barra superior. Está aceso enquanto a
janela está aberta; clique nele de novo, ou no botão de fechar da janela, para
a fechar. A janela flutua sobre os leitores e a cartucheira e não os bloqueia:
nada nela pára o áudio nem os atalhos da janela principal.

Mova-a pela sua barra de título e redimensione-a pelas margens e cantos. A
barra de título tem três botões: **Minimizar** (só fica a barra de título,
onde estava a janela, ou no topo da área principal quando a janela está
maximizada; clique de novo para expandir), **Maximizar** (a janela preenche
a área principal; **Restaurar** devolve-a) e **Fechar**. Uma janela que não
coubesse, depois de a janela principal encolher ou passar para um ecrã mais
pequeno, é trazida de volta para dentro da área principal.

A posição e o tamanho da janela, o estado minimizado ou maximizado, a largura
da árvore, as colunas, a ordenação e a categoria selecionada são guardados
com a sessão e regressam no arranque seguinte. Se a janela está aberta
também é guardado, mas nada toca por si só por causa disso. O texto do campo
de pesquisa não é guardado.

## Categorias {#categories}

A árvore à esquerda agrupa as faixas. No topo, **Todas as faixas**; depois as
suas categorias, que podem ter subcategorias; no fim, **Sem categoria**, as
faixas que não pertencem a nenhuma categoria. Cada item mostra a sua cor e uma
contagem: uma categoria conta as suas faixas e as das suas subcategorias. Uma
faixa pertence a uma só categoria, no máximo.

- **Clique** num item para listar as suas faixas na tabela. Uma categoria
  lista também as faixas das suas subcategorias.
- A **seta** antes de uma categoria abre ou fecha as suas subcategorias. As
  que estão abertas são guardadas.
- **Nova categoria** (＋ por cima da árvore) pede um nome, uma cor e um
  *tipo*. Uma categoria nova assume a cor e o tipo da categoria em que é
  criada; o tipo é o que uma faixa assume quando entra na categoria, e
  alterá-lo depois deixa as faixas que já estão nela como estão. Um nome não
  pode ser vazio nem repetir o nome de uma irmã; as categorias só se
  aninham até um limite.
- **Clique com o botão direito** numa categoria (ou prima longamente num ecrã
  tátil) para **Nova subcategoria…**, **Mudar o nome…**, **Cor**, **Tipo** e
  **Eliminar…**. No computador o menu tem também **Vigiar pasta…** (**Alterar
  pasta vigiada…** e **Deixar de vigiar a pasta** quando já vigia uma) e
  **Remover faixas em falta…**; ver [Pastas vigiadas](#watched-folders) e
  [Remover faixas](#removing-tracks).
- **Arraste** uma categoria para a mover: para cima de outra categoria para a
  aninhar, ou para o espaço entre duas categorias para a pôr aí. Uma
  categoria não pode ir para dentro de si própria. Num ecrã tátil a árvore só
  aceita toques e desloca-se sob um dedo; uma categoria não pode ser arrastada
  aí.

**Eliminar…** nomeia o que sai com a categoria: as suas subcategorias e todas
as faixas nelas são removidas da biblioteca. Os ficheiros nunca são
apagados. Se alguma dessas faixas estiver em uso, uma segunda pergunta diz
quantas entradas de listas e cartuchos as usam: **Remover mesmo assim** remove
as entradas das suas listas e esvazia os cartuchos. Nada é removido enquanto
alguma estiver no ar; um aviso diz o que está a tocar.

A mesma árvore está em **Definições → Biblioteca**, para criar, mudar o nome,
recolorir, mudar o tipo, mover e eliminar categorias sem abrir a janela (não
tem *Todas as faixas*, *Sem categoria* nem seleção).

## Pastas vigiadas {#watched-folders}

No computador, uma categoria pode vigiar uma pasta. Clique com o botão direito
nela e escolha **Vigiar pasta…**, e depois escolha a pasta. A categoria mostra
uma marca ⟳ antes da sua contagem; passe o rato por cima para ler a pasta.

- Cada ficheiro de áudio da pasta e das suas subpastas entra na categoria. As
  subpastas não criam subcategorias.
- Uma faixa que já está noutra categoria fica lá. Uma faixa que não está em
  nenhuma categoria entra nesta. Definir, alterar ou deixar de vigiar nunca
  move uma faixa: as faixas de uma pasta que já não é vigiada ficam na sua
  categoria.
- Enquanto a aplicação corre, a pasta é acompanhada: um ficheiro novo entra
  depois de estar quieto por um momento (`library.watch_debounce_ms`, 1
  segundo por predefinição, para que um ficheiro que ainda está a ser copiado
  não seja lido a meio). A pasta também é analisada sempre que a aplicação
  arranca, para apanhar o que mudou enquanto esteve fechada.
- Um ficheiro que desaparece da pasta (apagado, movido ou renomeado) deixa a
  sua faixa na biblioteca, marcada como em falta, com os seus marcadores. Se
  o ficheiro voltar, a faixa volta a ser normal por si só. Um ficheiro
  renomeado entra como uma faixa nova com o seu novo nome.
- As pastas vigiadas podem estar aninhadas. Um ficheiro sob duas delas vai
  para a categoria da mais interior; deixar de vigiar uma delas deixa a outra
  acompanhada. A mesma pasta não pode ser vigiada por duas categorias: a
  aplicação diz que categoria já a tem.
- Nada do que uma pasta vigiada faz carrega ou toca uma faixa.

Se a pasta não puder ser acompanhada, a marca ⟳ muda de cor e a sua dica diz
porquê: *Pasta vigiada não encontrada: …. Será procurada de novo.* (por
exemplo, uma unidade que não está montada), *Não é possível ler a pasta
vigiada: …. Será tentada de novo.*, ou *As alterações em … não são
acompanhadas; a pasta é analisada ao iniciar e tentada de novo.* (o sistema
recusou vigiá-la). Uma pasta guardada como um caminho que não está completo
(editada à mão) não é vigiada de todo, e a marca di-lo.

A cada 30 segundos (`tuning.missing_recheck_ms`) a aplicação verifica que
cada pasta vigiada continua lá, por isso uma unidade desligada enquanto corre
também é comunicada. Uma pasta com problemas é tentada de novo a esse ritmo e
analisada assim que regressa. Uma pasta que não está lá, não pode ser lida, ou
está vazia enquanto a biblioteca ainda tem faixas dela (uma unidade ou
partilha que não está montada costuma parecer uma pasta vazia) conta como não
encontrada. A vigilância não marca então nenhuma das suas faixas como em falta
e não remove nada, também quando a pasta está dentro de outra pasta vigiada.

Se esvaziar uma pasta vigiada de propósito, o aviso fica até haver ficheiros
nela de novo ou deixar de a vigiar, e as suas faixas ficam na biblioteca sem
serem marcadas como em falta. Para as limpar, selecione-as na tabela e
escolha **Remover da biblioteca…**, ou escolha **Voltar a analisar** nelas: a
análise marca como em falta as faixas cujo ficheiro desapareceu, e **Remover
faixas em falta…** remove-as então (se a pasta ainda é vigiada, o seu aviso
desaparece na verificação seguinte).

Uma exceção à regra acima: uma faixa que nunca foi analisada continua a ser
marcada como em falta quando a análise não encontra o seu ficheiro, onde quer
que esteja.

Uma subpasta que não pode ser lida é saltada e o registo di-lo; as ligações
de pastas (ligações simbólicas para pastas) não são seguidas.

Os ficheiros novos vão para a análise atrás de tudo o que um leitor ou um
cartucho precisa, por isso uma pasta com milhares de ficheiros demora; o
progresso da análise mostra-o.

O mesmo menu e a mesma marca estão em **Definições → Biblioteca**. Não estão
no Android, que não vigia pastas: aí uma categoria pode obter os seus
ficheiros de uma pasta do dispositivo que relê quando escolher (ver [Android →
Biblioteca](android.md#library)).

## A tabela de faixas {#the-track-table}

A tabela lista as faixas do item selecionado, nas mesmas colunas das tabelas
das listas (as escolhidas em **Definições → Listas → Colunas da tabela**, sem
a posição `#`) seguidas de **Categoria** e **Tipo**. Uma célula estreita à
esquerda assinala uma faixa cujo ficheiro está em falta ou não pode ser lido.
Uma faixa à espera da sua análise é assinalada à direita da sua célula
**Título** (por isso só quando a coluna Título está visível).

- Arraste o separador de um cabeçalho para alterar a largura de uma coluna;
  clique num cabeçalho para ordenar por ele e de novo para inverter. As
  larguras e a ordenação são guardadas.
- **Clique** seleciona uma linha, **Ctrl** (**Cmd** no macOS) **clique**
  acrescenta ou remove uma, **Shift clique** seleciona as linhas entre o
  último clique e este. A seleção só guarda linhas que estão listadas: uma
  linha que sai da tabela, por causa de uma pesquisa, de outra categoria ou de
  uma remoção, sai da seleção. Cada ação aplica-se às linhas selecionadas pela
  ordem da tabela.
- **Adicionar ficheiros** (por cima da tabela) escolhe ficheiros de áudio e
  adiciona-os à biblioteca, na categoria selecionada (sem categoria quando
  *Todas as faixas* ou *Sem categoria* está selecionada).

Num ecrã tátil as linhas só aceitam toques e deslocam-se sob um dedo; um
toque longo abre o menu, e as faixas são enviadas com os menus (abaixo), não
arrastando.

## Pesquisa {#search}

O campo por cima da tabela encontra faixas por texto. Procura no título,
artista, álbum, artista do álbum, compositor, género, comentário e nome do
ficheiro, sem distinguir maiúsculas nem acentos, e uma faixa é listada quando
contém todas as palavras que escreveu (por qualquer ordem e em qualquer
desses campos). Pesquisa no item selecionado; ative **Toda a biblioteca** para
pesquisar todas as faixas seja qual for a seleção. O âmbito é guardado; o
texto não. Limpe o campo para ver de novo o item inteiro.

## O menu da linha {#the-row-menu}

Clique com o botão direito numa linha. Se a linha não está selecionada,
passa a ser a seleção; caso contrário, o menu atua em todas as linhas
selecionadas. A primeira linha mostra o título da faixa quando o menu é para
uma faixa, e quantas faixas estão selecionadas quando é para várias.

| Item | Ação |
|---|---|
| Adicionar a ▸ | Uma lista: as faixas são adicionadas ao seu fim |
| Adicionar a um cartucho ▸ | Um cartucho da página visível da cartucheira: as faixas preenchem o cartucho escolhido e os cartuchos vazios seguintes da página, pela ordem da grelha. As faixas que não cabem ficam de fora e um aviso conta-as |
| Mover para a categoria ▸ | Uma categoria, ou Sem categoria |
| Tipo ▸ | Música, Jingle, Efeito, Anúncio ou Voz |
| Editar etiquetas… | O editor de etiquetas (uma faixa) |
| Loudness… | A janela de loudness (uma faixa) |
| Voltar a analisar | Analisar as faixas de novo |
| Repor os marcadores automáticos | Descartar os marcadores manuais |
| Mostrar a localização do ficheiro | Abrir a pasta do ficheiro (uma faixa) |
| Remover da biblioteca… | Remover as faixas da biblioteca, depois de perguntar (abaixo) |

## Pré-escuta {#pre-listening}

A barra por baixo da barra de título pré-escuta a faixa selecionada na
própria saída **Biblioteca** da biblioteca (escolhida em [Definições → Saídas
de áudio](settings.md#audio-outputs)), por exemplo uns auscultadores. Um
telemóvel pré-escuta a partir da sua lista de faixas (ver [Num
telemóvel](#on-a-phone)).

A barra mostra:

- **Pré-escutar**, **Pausar a pré-escuta** e **Parar a pré-escuta**;
- o tempo restante da faixa, em grande, o seu título, artista e categoria;
- um pequeno medidor à direita. Lê apenas esta pré-escuta, nunca o medidor de
  um leitor ou da cartucheira; clique nele para reiniciar o seu máximo;
- a forma de onda do ficheiro inteiro, com os seus marcadores, e uma linha
  `decorrido / total` por baixo.

**Pré-escutar** começa a faixa selecionada a partir do seu cue-in e toca-a
até ao fim do ficheiro. **Pré-escutar** numa pré-escuta em pausa retoma-a, e
numa que está a tocar não faz nada: **Parar a pré-escuta** e depois
**Pré-escutar** começa de novo a partir do cue-in. A normalização do loudness
aplica-se como no CUE de um leitor.

A forma de onda funciona como a da janela de CUE de um leitor: clique para
saltar (um clique ou a libertação do rato salta; arrastar não), aproxime e
desloque-a com a roda, e edite os marcadores e abra **Loudness…** a partir do
seu menu (ver [Editar marcadores](markers-and-mixing.md#editing-markers)).
Enquanto a pré-escuta está parada, um clique define antes onde o próximo
**Pré-escutar** começa.

**A pré-escuta segue a seleção.** Com várias linhas selecionadas, a última
em que clicou é a que usa.

- Enquanto toca, selecionar outra linha passa para essa faixa a partir do seu
  cue-in e continua a tocar, sem perguntar.
- Enquanto está em pausa, a faixa nova é carregada em pausa.
- Enquanto está parada, continua parada e a barra mostra a faixa nova.
- Não selecionar nada (nenhuma linha selecionada, ou a linha selecionada
  filtrada) deixa como está uma pré-escuta em curso.
- Selecionar uma faixa que não pode ser tocada (o seu ficheiro está em falta
  ou é ilegível) pára a pré-escuta, e a barra diz que o ficheiro não pode ser
  reproduzido.

**O que a pára.** **Parar a pré-escuta**; remover a sua faixa da biblioteca;
fechar a janela (minimizá-la não). Nunca toca depois de um reinício.

**Sem saída Biblioteca** (no computador; o Android toca antes na saída
predefinida, ver [Android → Biblioteca](android.md#library)) o **Pré-escutar**
fica esbatido (passe o rato por cima para ver o motivo) e uma linha ao lado
diz que a pré-escuta precisa de uma saída CUE da biblioteca, com uma ligação
que abre Definições → Saídas de áudio na linha **Biblioteca**. A barra
continua a mostrar a faixa, a sua forma de onda e os seus marcadores.

**Numa saída do ar.** A saída da Biblioteca pode ser uma saída do ar (o mesmo
dispositivo e canais que o Main de um leitor ou da cartucheira). Funciona
então como o CUE dos leitores e da cartucheira (ver [CUE
(pré-escuta)](players.md#cue-pre-listen)): enquanto algo toca aí, o
**Pré-escutar** pergunta primeiro (**Pré-escutar mesmo assim** ou
**Cancelar**), e uma pré-escuta que não confirmou pára quando algo começa
nessa saída.

## Arrastar para listas e cartuchos {#dragging-to-playlists-and-carts}

Arraste com o rato as linhas selecionadas (ou a linha de onde começa a
arrastar, quando não está selecionada):

- para um **separador de lista** de um leitor: as faixas vão para o fim dessa
  lista;
- para uma **tabela de lista**: as faixas vão para onde a linha violeta
  mostra;
- para um **cartucho**: o cartucho recebe a primeira faixa (um cartucho tem um
  só ficheiro), substituindo o seu ficheiro quando está parado. Um cartucho
  que está a tocar ou no CUE recusa o largar, porque um ficheiro novo
  pará-lo-ia no ar.

## Remover faixas {#removing-tracks}

**Remover da biblioteca…** pergunta primeiro: os ficheiros não são apagados.
Quando alguma das faixas está em uso, uma segunda pergunta conta as entradas
de listas e os cartuchos que as perdem. Uma faixa que está no ar (a tocar num
leitor ou na cartucheira) não pode ser removida: nada é removido e um aviso
diz o que está a tocar.

**Remover faixas em falta…** (no menu da categoria, em qualquer categoria)
remove as faixas dessa categoria e das suas subcategorias cujo ficheiro está
em falta e que nenhuma entrada de lista ou cartucho usa. Pergunta primeiro,
com o seu número; nenhum ficheiro é tocado. As faixas em uso são mantidas, e
uma categoria sem nenhuma di-lo num aviso. Use-o para limpar depois de
ficheiros saírem de uma pasta vigiada: a aplicação nunca os remove por si só.

## A partir de uma lista {#from-a-playlist}

No computador, o menu de faixa de uma linha de lista tem **Mostrar na
biblioteca** e **Mover para a categoria ▸**, a seguir a **Loudness…**.
*Mostrar na biblioteca* abre a janela na categoria da faixa e seleciona a sua
linha; *Mover para a categoria* arruma a faixa sem abrir a janela. Para uma
faixa que não está na biblioteca, *Mostrar na biblioteca* aparece
desativado e *Mover para a categoria* não aparece.

## Num telemóvel {#on-a-phone}

Um telemóvel não tem janela da biblioteca: o seu separador **Biblioteca** abre
nas categorias (*Todas as faixas*, as suas categorias, *Sem categoria* e
**Música do dispositivo**), e um toque numa lista as suas faixas com uma
pesquisa e **Toda a biblioteca**. Assinale faixas para as adicionar a uma
lista, toque longamente numa linha para o seu menu, pré-escute uma faixa com o
seu 🎧, e adicione a música do dispositivo a uma categoria com o **＋** da sua
lista. No Android, sem saída da Biblioteca, a pré-escuta toca na saída
predefinida. Ver [Android → Biblioteca](android.md#library).

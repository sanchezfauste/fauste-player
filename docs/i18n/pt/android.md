# Android

O Fauste Player também corre em telemóveis e tablets Android (Android 10 ou
posterior), sobre o mesmo núcleo de emissão do computador: os leitores, as
listas, a cartucheira, os marcadores, a normalização do loudness e todas as
regras de reprodução funcionam da mesma forma.

## Instalação {#install}

Uma versão que inclui a aplicação tem `fauste-player-<version>-android.apk`
na sua página **Releases** (com o seu `.sha256`). Abra-o no telemóvel e
permita que o navegador ou o gestor de ficheiros instale aplicações quando o
Android perguntar. Para atualizar, instale o APK mais recente da mesma forma:
as suas listas e definições ficam. A aplicação ainda não está no Google Play.

## Primeiro arranque {#first-start}

- **Acesso à música.** Permita o acesso à música (e às notificações, no
  Android 13 e posterior). Sem ele, o navegador de música fica vazio.
- **A emissão continua.** Enquanto um leitor ou um cartucho toca, fica uma
  notificação visível e o áudio continua com o ecrã apagado ou a aplicação em
  segundo plano. Como no computador, nada vai para o ar por si só depois de um
  arranque ou de uma falha.
- **Voltar e fechar.** Voltar fecha primeiro o que está aberto (o navegador
  de música, as Definições, a janela Acerca de, um leitor aberto, uma página
  do separador Biblioteca). Sem mais nada a fechar, Voltar guarda a aplicação
  enquanto algo soa (um leitor, um cartucho ou uma pré-escuta CUE) e continua
  a tocar; quando nada soa, guarda e fecha a aplicação. Deslizar a aplicação
  para fora no seletor de aplicações faz o mesmo: um deslize nunca pára o que
  está a tocar, e a aplicação fecha quando fica em silêncio.
- **O som de outras aplicações.** Quando outra aplicação toma o áudio, o
  Fauste Player continua a tocar e di-lo na barra de estado em baixo.

## O ecrã {#the-screen}

Num **telemóvel** a barra superior tem quatro separadores, **Direto**,
**Cartucheira**, **Listas** e **Biblioteca**, mostrados como ícones (o
escolhido também com o seu nome quando há espaço), e depois o **cadeado** e
**⋮** (Mais: Definições e Acerca de). O relógio aparece quando há espaço para
ele. Com o telemóvel de lado, o ecrã Listas põe os seus botões de leitor (P1,
P2…) na barra superior, e a barra de estado em baixo só aparece quando tem
algo a dizer.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Direto num telemóvel: dois leitores como cartões, um inteiro e o início do seguinte, e a cartucheira por baixo">
<img src="../../images/android/2-carts.png" width="180" alt="Cartucheira num telemóvel: a página Geral na vertical, dois cartuchos por linha">
<img src="../../images/android/3-playlists.png" width="180" alt="Listas num telemóvel: os botões P1 a P4, os separadores de listas, a lista de faixas e o cartão do leitor em baixo">
<img src="../../images/android/4-music.png" width="180" alt="Música do dispositivo num telemóvel: a pasta Demo com três canções assinaladas e a barra para as adicionar à lista Manhã do P1">
</p>

- *Direto:* cada leitor como um cartão, e a cartucheira. Cada cartão tem, de
  cima para baixo, o nome do leitor, o estado, Single/Cont e CUE; a capa, o
  título e o artista e a faixa seguinte; a forma de onda (um toque salta);
  depois o tempo decorrido e a duração da faixa com o tempo restante em
  dígitos grandes sobre o transporte (Play / Seguinte, Anterior, Pausa, Stop,
  Reiniciar, Stop com fade, Stop no fim da faixa atual) com botões do tamanho
  de um dedo, e um medidor fino com o fader de volume (o nível do fader
  aparece por baixo). O tempo restante acaba onde acabam os botões, mais
  pequeno para uma faixa longa. Na maioria dos telemóveis o medidor mostra a
  sua escala à esquerda, e num mais largo (cerca de 406 dp ou mais) o fader
  também mostra os seus níveis; num telemóvel estreito (menos de cerca de
  384 dp) nenhum dos dois o faz, e os botões ficam um pouco mais pequenos
  para caberem. Os botões funcionam enquanto outro dedo segura o fader, e um
  toque conta mesmo que o dedo role um pouco ou fique um momento em baixo. Na
  vertical, os cartões ficam em cima, um cartão inteiro e o início do
  seguinte (desloque para ver mais), e a cartucheira por baixo. De lado, cada
  leitor é uma linha à esquerda (o seu nome e estado, o título, o tempo
  restante, Play / Seguinte, Pausa e Stop; toque na linha noutro sítio para
  abrir o leitor em Listas) e a cartucheira fica à direita. Toque em
  **CARTUCHEIRA** no seu cabeçalho para a dobrar: só fica o cabeçalho, em
  baixo (os separadores de página e *Parar tudo* continuam a funcionar), e os
  leitores ficam com o espaço; toque de novo para a desdobrar. O separador
  Cartucheira mostra sempre a cartucheira inteira. Uma página de cartuchos
  demasiado larga para o espaço aparece com menos colunas, pela mesma ordem.
- *Cartucheira:* a página de cartuchos visível, a preencher o ecrã. Deslize
  para os lados para mostrar a página seguinte ou a anterior. Na vertical,
  uma página mais larga do que alta é rodada (8 × 2 aparece como 2 × 8), pela
  mesma ordem.
- *Listas:* os botões no topo (P1, P2…) escolhem o leitor; por baixo estão os
  seus separadores de listas (+ cria uma nova), a lista de faixas e, em baixo
  (à direita quando o telemóvel está de lado), o cartão do leitor com uma
  forma de onda maior para preparar uma faixa (fica mais baixo quando falta
  espaço, para que os botões apareçam sempre). Um toque longo numa linha abre
  o seu menu, que também tem **Subir**, **Descer** e **Mover para outro
  leitor** (a faixa vai logo a seguir à faixa seguinte desse leitor, na lista
  que ele mostra; a faixa no ar fica). Arraste uma linha pela sua pega ≡ para
  a mover. O rodapé tem **Adicionar** e, atrás de **⋯**, **Repor tocadas**.
  Um toque longo na forma de onda põe um marcador nesse ponto (cue-in, fim da
  intro, início do outro, MIX, cue-out) ou remove um que tenha posto; os
  marcadores só se editam aqui, não em Direto.
- *Biblioteca:* a biblioteca de faixas e a música do dispositivo (ver
  [Biblioteca](#library)).

Voltar a partir de outro ecrã vai para Direto; na Biblioteca volta primeiro à
lista de categorias (em Música do dispositivo, primeiro sobe uma pasta).

Quando nada se move no ecrã, a aplicação desenha-o uma vez por segundo (para
o relógio) para poupar bateria; um medidor que ainda está a descer depois de
uma pausa ou de um stop mantém-na a desenhar até o medidor estar em baixo.

**O cadeado** protege contra toques acidentais durante um programa. Fechado
(o cadeado fechado e aceso), os faders de volume apenas mostram o nível e as
formas de onda nem saltam nem editam marcadores; ficam esbatidos. Um
movimento de pinça e um arrasto continuam a aproximar e a mover uma forma de
onda, e o Play, o transporte, os cartuchos e as listas funcionam sempre. O
bloqueio mantém-se depois de um reinício e é guardado com a sessão, por isso
carregar outra sessão assume o bloqueio dessa sessão; a API remota e o MIDI
não são bloqueados.

Um arrasto que começa num fader de volume move-o; um que começa noutro sítio
desloca, por isso deslocar nunca altera um nível no ar. Um toque numa forma
de onda salta para aí, uma pinça de dois dedos sobre ela aproxima em torno
dos dedos, um arrasto com um dedo move uma vista ampliada e **Vista
completa** mostra de novo a faixa inteira. Numa lista, um dedo desloca; para
mover uma faixa, use a sua pega ≡ ou o menu da linha.

Num **tablet** a disposição é a do computador (ver [Disposição](layout.md)),
com alvos mais fáceis de acertar.

Um toque longo abre o que um clique com o botão direito abre no computador: o
menu da linha, os menus dos cabeçalhos, o menu de um cartucho (**Editar…**
escolhe o ficheiro de um cartucho vazio). Num telemóvel, as Definições
preenchem o ecrã e listam as suas secções; toque numa para a abrir, com o nome
de cada definição por cima do seu controlo. A seta ou Voltar regressa à
lista, e Voltar de novo (ou **Fechar**) fecha as Definições. Editar um
cartucho abre diretamente a sua secção.

Acerca do Fauste Player → **Licenças de terceiros** abre os avisos das
bibliotecas com que a aplicação é construída num navegador ou visualizador
HTML.

## Biblioteca {#library}

Num **telemóvel** o separador **Biblioteca** abre numa lista de categorias:
**Todas as faixas**, as suas categorias (cada subcategoria sob a sua
principal, com a sua cor e quantas faixas tem), **Sem categoria** e **Música
do dispositivo**. A biblioteca é a mesma que a [janela da biblioteca](library.md)
do computador mostra: cada ficheiro adicionado a uma lista, a um cartucho ou
à biblioteca está nela. As categorias são criadas, renomeadas, recoloridas e
eliminadas em **Definições → Biblioteca**.

**As faixas de uma categoria.** Toque numa categoria para abrir as suas
faixas (com as das suas subcategorias). A seta no seu cabeçalho, ou Voltar,
regressa às categorias. Mudar de separador e voltar encontra a lista como a
deixou. Abrir uma categoria a partir das categorias começa com uma pesquisa
vazia. A pesquisa filtra a lista por título, artista, álbum e as outras
etiquetas que a janela da biblioteca pesquisa; **Toda a biblioteca**
pesquisa todas as categorias. Cada linha mostra o título (ou o nome do
ficheiro), o artista, a duração e as marcas da janela para um ficheiro em
falta, ilegível ou ainda não analisado.

Toque nas linhas para as assinalar. Enquanto algo está assinalado, aparece em
baixo a barra de Música do dispositivo: quantas, o leitor e a lista para onde
vão (toque nela para escolher outra), **Adicionar no fim** e **Adicionar
após a seguinte**, como em Música do dispositivo. As faixas vão pela ordem em
que as assinalou, e adicionar limpa as marcas. Voltar às categorias também as
limpa.

**O menu da linha.** Um toque longo numa linha abre o seu menu: **Adicionar a
▸** (a lista de um leitor, no fim), **Pré-escutar**, **Mover para a
categoria ▸**, **Voltar a analisar** e **Remover da biblioteca…**, que
pergunta como no computador (ver [Remover faixas](library.md#removing-tracks)).
Enquanto há linhas assinaladas, um toque longo noutra linha assinala-a
também e o menu atua em todas as linhas assinaladas, pela ordem em que as
assinalou; o **Pré-escutar** não é então oferecido. As marcas ficam depois da
ação do menu.

**Pré-escuta.** Os 🎧 à direita de uma linha tocam a sua faixa a partir do
seu cue-in na saída de pré-escuta da biblioteca; os 🎧 de outra linha passam
para essa faixa. Uma faixa fina por cima da lista mostra o título, o tempo
restante, **Pausa**/**Play** e **Stop** enquanto toca (ou está em pausa), e
uma dica caso contrário: mantém sempre o seu lugar, por isso as linhas não se
movem quando uma pré-escuta começa ou pára. Os 🎧 ficam esbatidos quando o
ficheiro está em falta ou não pode ser tocado. A pré-escuta pára quando a sua
faixa sai da biblioteca e quando sai do separador Biblioteca (para outro
separador, ou por Voltar); nas categorias ou em Música do dispositivo
continua, mas a sua faixa só aparece na lista de faixas.

Ao contrário do computador, o Android (um telemóvel ou um tablet) toca a
pré-escuta da biblioteca na saída predefinida quando não há saída escolhida
para ela: em **Definições → Saídas de áudio → Biblioteca** a escolha vazia lê-se
*Predefinido do sistema*, e o seu **Testar Cue** toca aí. Escolha aí outra
saída (uns auscultadores com fio, por exemplo) para pré-escutar longe do ar. A
saída predefinida costuma ser também a dos leitores e a da cartucheira, por
isso a pré-escuta pode ouvir-se no ar: enquanto algo toca nessa saída, os 🎧
perguntam primeiro (**Pré-escutar mesmo assim** ou **Cancelar**), e uma
pré-escuta que não confirmou pára quando algo começa aí, como o CUE de um
leitor (ver [CUE (pré-escuta)](players.md#cue-pre-listen)).

**Adicionar música do dispositivo a uma categoria.** O **＋** no cabeçalho da
lista de faixas abre a Música do dispositivo para essa categoria. A sua barra
tem então apenas **Adicionar a <categoria>** (**Adicionar a Sem categoria** a
partir de *Todas as faixas* ou *Sem categoria*), que adiciona os ficheiros
assinalados à biblioteca nessa categoria e volta à lista; um ficheiro que já
está na biblioteca junta-se a ela se não tem categoria e mantém a que tem
caso contrário, como no computador. A Música do dispositivo mantém as suas
marcas entre visitas, por isso os ficheiros assinalados numa visita anterior
também são adicionados: verifique primeiro a contagem na barra. **Voltar à
lista**, no topo, ou Voltar regressa à lista sem adicionar. A Música do
dispositivo aberta a partir das categorias tem antes **Voltar às categorias**
no topo.

**A pasta do dispositivo de uma categoria.** Uma categoria pode obter os seus
ficheiros de uma pasta da música do dispositivo. No seu menu (um toque longo
nela em **Definições → Biblioteca**, ou na janela da biblioteca num tablet),
**Usar uma pasta do dispositivo…** abre a Música do dispositivo para escolher
a pasta (**Usar esta pasta**); o menu da categoria passa então a nomeá-la,
por cima dos seus itens de pasta. Nada vigia a pasta: quando copiou música
nova para lá, toque no **↻** no cabeçalho da lista de faixas da categoria (ou
em **Reler a pasta do dispositivo** no seu menu). Cada ficheiro que a Música
do dispositivo lista na pasta e nas suas subpastas é adicionado à categoria,
com as regras acima (um ficheiro que já está noutra categoria mantém-na), e um
aviso diz quantos ficheiros foram lidos. Os ficheiros que saíram da pasta
ficam na biblioteca; **Voltar a analisar** marca como em falta aqueles cujo
ficheiro desapareceu. **Alterar a pasta do dispositivo…** e **Esquecer a
pasta do dispositivo** estão no mesmo menu; uma pasta pertence a uma só
categoria.

### Música do dispositivo {#device-music}

A **Música do dispositivo** mostra os ficheiros de áudio do dispositivo por
**Pastas** (com o caminho por cima da lista; toque numa parte dele para subir),
**Artistas** ou **Álbuns**, e uma pesquisa por título, artista, álbum e nome
do ficheiro. Cada linha mostra o título, o artista e a duração; toque nas
linhas para as assinalar. Enquanto algo está assinalado, uma barra em baixo
mostra quantas, o leitor e a lista para onde vão (toque nela para escolher
outra), **Adicionar no fim** e, quando a faixa seguinte desse leitor está
nessa lista, **Adicionar após a seguinte**. As faixas vão pela ordem em que as
assinalou. Um ficheiro do dispositivo não tem pré-escuta.

**Adicionar** no rodapé de uma lista, o ficheiro de um cartucho e Definições →
Listas → Pasta de música abrem a Música do dispositivo num telemóvel (num
tablet, a janela do **navegador de música**, por pastas). Assinale ficheiros e
toque em **Adicionar**, toque num ficheiro para um cartucho, ou em **Usar esta
pasta**; **Cancelar** ou Voltar regressa a onde estava. Copie música para o
dispositivo (por exemplo, para a sua pasta *Música*) para a ver aí. O Fauste
Player só lê a sua música: as etiquetas e as capas não são escritas, e as
listas e as páginas de cartuchos ainda não são importadas nem exportadas no
Android.

Num **tablet** a biblioteca é a [janela da biblioteca](library.md) do
computador, aberta a partir da barra superior.

## Saídas {#outputs}

Definições → Saídas de áudio lista as saídas do telemóvel por tipo: o
altifalante, uns auscultadores com fio, uma interface USB, uma coluna
Bluetooth («JBL Flip (Bluetooth)»). Cada leitor, a cartucheira e os seus CUE
escolhem uma, como no computador, e a pré-escuta da biblioteca também (ver
[Biblioteca](#library)). Uma coluna Bluetooth que é desligada e ligada de novo
é encontrada outra vez pela sua rota; enquanto está ausente, os seus leitores
comportam-se como com um dispositivo em falta no computador. Se duas saídas
podem soar ao mesmo tempo depende do telemóvel.

## Não existe no Android {#not-on-android}

As superfícies de controlo MIDI, o controlo remoto, a edição de etiquetas e
capas e os ficheiros de listas e de páginas de cartuchos são, por agora,
funcionalidades do computador. Um teclado ligado a um tablet funciona como no
computador (ver [Teclado](keyboard.md)).

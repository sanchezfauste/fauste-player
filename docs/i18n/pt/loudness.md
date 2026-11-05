# Normalização do loudness

As faixas vêm de muitas fontes e são masterizadas a níveis muito diferentes:
um single pop recente pode soar duas vezes mais alto do que uma gravação dos
anos 70 ou um promo falado. A **normalização do loudness** toca cada faixa e
cada cartucho com o mesmo loudness percebido, para que o operador não tenha
de andar com o fader de uma faixa para a seguinte.

Está **ativada nas instalações novas**; uma configuração existente mantém a
sua definição atual (toda a configuração guardada já a armazena). Altere-a em
[Definições → Loudness](settings.md#loudness).

![Definições, Loudness: a normalização ativada, com o objetivo, o teto de true peak, o ganho máximo e o ajuste máximo nas predefinições](../../images/guide/settings-loudness.png)

## Como funciona {#how-it-works}

**Um ganho fixo por faixa.** Cada faixa recebe um só ganho, em dB, decidido
quando a faixa é carregada e mantido até parar. Nada é comprimido nem
limitado: a faixa inteira fica mais alta ou mais baixa na mesma medida, por
isso a sua própria dinâmica fica como foi masterizada. O fader do leitor
atua por cima dele, como antes.

**A medição.** A análise em segundo plano mede cada ficheiro uma só vez:

- o seu **loudness integrado**, em LUFS: quão alto soa o ficheiro inteiro,
  como o definem a ITU-R BS.1770 e a EBU R128 (as passagens baixas e o
  silêncio são descartados, por isso um fade out longo ou uma pausa não o
  baixam);
- o seu **true peak**, em dBTP: o nível mais alto que o sinal atinge entre
  amostras depois de convertido para analógico.

Os ficheiros novos são medidos com a sua primeira análise. Os ficheiros
adicionados antes de o loudness ser medido são medidos por uma **passagem em
segundo plano** que corre enquanto a análise normal não tem nada a fazer, sem
tocar nos seus marcadores. Corre com a normalização ativada ou desativada,
para que as medições estejam prontas quando a ativar. **Voltar a analisar**
(no menu da lista) mede um ficheiro de novo, lendo-o mesmo que o tamanho e a
data não tenham mudado. Um ficheiro com menos de 0,4 s nunca é medido, e um
ficheiro silencioso não tem loudness: ambos tocam sem alterações.

**O ganho.** O ganho leva o loudness medido ao **Objetivo**, mais o
**Ajuste** da própria faixa (ver [Por faixa](#per-track-exclude-and-trim)):

> ganho = objetivo − loudness medido + ajuste

Uma faixa mais alta do que o objetivo é baixada o que for preciso. Uma faixa
mais baixa é aumentada, mas nunca mais do que o **Ganho máximo**, e nunca
tanto que o seu true peak passasse o **Teto de true peak**: não há limitador,
por isso o teto é o que impede uma faixa aumentada de saturar. Uma faixa cujo
true peak já está acima do teto não é aumentada de todo.

Exemplos, com as predefinições (objetivo −16 LUFS, teto −1 dBTP, ganho
máximo 12 dB):

| Faixa | Medido | True peak | Ganho |
|---|---|---|---|
| Uma master moderna alta | −9.0 LUFS | +0.5 dBTP | −7.0 dB |
| Uma master antiga mais baixa | −20.0 LUFS | −10.0 dBTP | +4.0 dB |
| Uma faixa baixa com picos altos | −22.0 LUFS | −3.0 dBTP | +2.0 dB (limitado pelo teto) |
| Uma gravação muito baixa | −35.0 LUFS | −20.0 dBTP | +12.0 dB (limitado pelo ganho máximo) |

**Quando o ganho é decidido.** Quando uma faixa é carregada: preparada como
seguinte de um leitor, iniciada, carregada em pausa, pré-escutada no CUE ou
disparada como cartucho. Uma faixa que está a tocar ou em pausa **mantém o
seu ganho** até parar, através de saltos e pausas. Alterar as definições, a
medição de uma faixa ou o seu ajuste aplica-se da próxima vez que a faixa for
carregada; a faixa seguinte que já espera num leitor é preparada de novo com
o novo ganho, por isso começa com ele.

**Pré-escuta.** O CUE e a pré-escuta de cartuchos tocam uma faixa com o mesmo
ganho que ela recebe no ar, por isso o que ouve nos auscultadores
corresponde ao programa.

**Medidores.** Os medidores e a sua leitura de loudness medem depois do
ganho, por isso mostram o nível normalizado. O **Objetivo de loudness** dos
próprios medidores (Definições → Medidores) só colore a leitura; não altera
nenhum ganho.

## Saídas bit-perfect {#bit-perfect-outputs}

Um leitor ou cartucho cuja saída é [bit-perfect](bit-perfect.md) **nunca é
normalizado**: toca sempre com ganho unitário (0 dB), sejam quais forem as
definições, porque qualquer ganho alteraria as amostras que o dispositivo
recebe. Com a normalização ativada, o seu cabeçalho mostra um indicador
**0.0 dB** âmbar cuja dica diz «Saída bit-perfect: as faixas tocam sem
alterações.» O mesmo vale para a pré-escuta num dispositivo Cue bit-perfect,
enquanto a saída Main do leitor, se não for bit-perfect, continua a receber o
ganho.

Uma faixa que já está a tocar (ou em pausa) quando o seu dispositivo se torna
bit-perfect mantém o ganho com que começou, e o indicador **BP** fica apagado
para ela. A faixa seguinte toca a 0 dB e em bit-perfect. Uma faixa DSD nessa
situação toca convertida para PCM com o seu ganho, nunca como um fluxo DSD
sem alterações.

## O ganho no cabeçalho do leitor {#the-gain-in-the-player-header}

Enquanto um leitor toca ou está em pausa, um indicador à esquerda do **BP**
mostra o ganho da faixa no ar. O indicador tem uma borda da sua cor e tem
sempre a mesma largura. Passe o rato por cima para ver o motivo.

| Indicador | Significado |
|---|---|
| **+4.0 dB**, **−7.0 dB** (azul) | A faixa é normalizada com este ganho. A dica é a decomposição do ganho descrita abaixo. Quando as definições, a medição ou o ajuste mudaram depois de a faixa ser carregada, diz apenas que a faixa mantém o ganho fixado quando foi carregada, e mostra esse ganho. |
| **0.0 dB** (âmbar) | A faixa toca sem alterações. A dica é a mesma decomposição, e a sua linha **Ganho automático** diz porquê: ainda não tem medição (ou é demasiado curta para medir), a saída é bit-perfect, ou a faixa está excluída da normalização. |
| Sem indicador | A normalização estava desativada quando a faixa começou, a faixa é silenciosa, ou o leitor está parado. Um leitor bit-perfect também não mostra nenhum enquanto a normalização está desativada. |

O indicador mostra o ganho que o leitor pediu quando a faixa começou.

### A decomposição do ganho {#the-gain-breakdown}

Em todo o lado onde o programa explica o ganho de uma faixa mostra o mesmo
bloco de linhas, com a etiqueta à esquerda e o valor à direita: a dica do
indicador de ganho, a janela de loudness (por baixo dos seus controlos), o
editor de cartuchos e a dica da faixa na tabela da lista.

| Linha | Significado |
|---|---|
| **Loudness integrada** | O loudness medido da faixa, em LUFS (ou «sem medição»). |
| **True peak** | O seu true peak mais alto, em dBTP (só quando medido). |
| **Objetivo** | O loudness a que cada faixa é levada (Definições → Loudness). |
| **Ganho automático** | O objetivo menos o loudness medido, antes do ajuste e de qualquer limite, ou «nenhum» e porquê: excluída, saída bit-perfect, normalização desativada, ainda sem medição ou silenciosa. |
| **Ajuste** | O ajuste manual da faixa em uso, quando não é 0 (com o valor guardado e o máximo quando um **Ajuste máximo** reduzido o limita). |
| **Ganho pretendido** e **Limitado por** | Só quando o teto de true peak ou o ganho máximo seguraram o ganho: o ganho automático mais o ajuste (mostrado quando há ajuste) e o limite que o travou. |
| **Ganho resultante** | Sempre a última, sob uma linha e a negrito: o ganho com que a faixa toca. 0.0 dB quando não é normalizada. |

Com a normalização desativada, uma nota acima do ganho resultante diz que
nada muda no ar e que o ajuste se aplica quando for ativada.

## Por faixa: excluir e ajustar {#per-track-exclude-and-trim}

Algumas faixas não devem seguir o objetivo: um indicativo da estação
masterizado alto de propósito, um tom de teste, uma faixa cuja medição não
corresponde ao que soa (a fala costuma soar mais alta do que a música com os
mesmos LUFS). Cada faixa tem duas definições para isso:

- **Excluir da normalização:** a faixa toca sempre sem alterações.
- **Ajuste:** dB somados ao ganho da faixa, de − a + o **Ajuste máximo**
  (Definições → Loudness), em passos de 0,5 dB. Faça duplo clique no
  controlo deslizante para o pôr de novo a 0. O ajuste fica esbatido enquanto
  a faixa está excluída.

O controlo deslizante é um **ajuste**: um desvio por cima do ganho
automático, por isso lê 0 dB para todas as faixas que não editou. A janela e
o editor de cartuchos mostram, por isso, a [decomposição do
ganho](#the-gain-breakdown) com os seus controlos: a linha **Ganho
resultante** acompanha o controlo deslizante à medida que o move. Na janela,
os controlos vêm primeiro e a decomposição por baixo deles, para que o
controlo deslizante fique onde está enquanto a decomposição cresce ou encolhe.
Enquanto a faixa está a soar, a janela acrescenta que uma alteração se aplica
desde o seu próximo carregamento.

Onde os alterar:

- **Numa forma de onda:** clique com o botão direito na forma de onda de um
  leitor (ou da sua janela de CUE) e escolha **Loudness…**. A janela nomeia a
  faixa e mostra a sua medição. Continua a editar essa faixa mesmo quando o
  leitor passa para outra; feche-a com o seu botão **×**. Fecha-se sozinha se
  a faixa for removida.
- **Numa linha da lista:** clique com o botão direito na linha e escolha
  **Loudness…**: a mesma janela, para a faixa dessa linha.
- **No editor de cartuchos:** Definições → Cartucheira, clique num cartucho:
  os mesmos dois controlos estão por baixo de **Parar os outros cartuchos ao
  disparar**, com a medição por baixo da etiqueta.
- **Remotamente:** `PUT /api/v1/tracks/<id>/loudness` (ver
  [Controlo remoto](remote-control.md)).

As definições pertencem à faixa e são guardadas com ela. Uma alteração
aplica-se da próxima vez que a faixa for carregada, como qualquer outra
alteração.

**A dica da faixa** da tabela da lista mostra as etiquetas da faixa e, sob
uma linha, a mesma [decomposição do ganho](#the-gain-breakdown), calculada
para o leitor a que essa lista pertence (um leitor bit-perfect toca-a sem
alterações).

## Escolher as definições {#choosing-the-settings}

- **Objetivo.** −16 LUFS serve a maioria dos programas musicais no ar e em
  linha. −23 LUFS (EBU R128) ou −24 LUFS (ATSC A/85) deixam mais folga, mas as
  faixas baixas precisam então de aumentos maiores, que o teto e o ganho
  máximo podem travar. Escolha um objetivo próximo de quão alto o seu material
  já é.
- **Teto de true peak.** −1 dBTP deixa margem para a conversão e para os
  codificadores com perdas mais adiante. Baixe-o quando um codificador de
  stream satura.
- **Ganho máximo.** Impede que uma gravação muito baixa (ou quase toda
  silenciosa) seja levantada até ao ruído. 0 dB só baixa as faixas altas.
- **Ajuste máximo.** Até onde pode ir o ajuste por faixa. Reduzi-lo limita os
  ajustes já definidos sem os alterar, até voltar a editar o loudness dessa
  faixa: a edição guarda então o ajuste limitado.

## As normas por trás das predefinições {#standards-behind-the-defaults}

- **Medição.** Loudness integrado e true peak tal como definidos pela
  ITU-R BS.1770 (ponderação K, com gating).
- **Objetivo −16 LUFS.** Segue as recomendações da AES para streaming e
  distribuição em linha. A AES TD1008 (2021) substitui a anterior AES
  TD1004.1.15-10, que recomendava −16 a −20 LUFS para streaming. Não é um
  valor de radiodifusão: a EBU R128 usa −23 LUFS e a ATSC A/85 usa
  −24 LKFS. Use um destes quando uma especificação de entrega o exigir (a EBU
  R128 para a troca de programas de radiodifusão europeus, a ATSC A/85 para a
  televisão digital dos EUA), e −16 para um stream ou programa em linha.
- **Teto de true peak −1 dBTP.** O true peak máximo da EBU R128, também usado
  pelas recomendações da AES. Os codificadores com perdas podem exceder na
  descodificação, por isso um stream enviado por um codificador de baixo
  débito pode querer um teto mais baixo, como −2 dBTP (uma prática comum, não
  parte dessas normas).
- **Ganho máximo e ajuste máximo, 12 dB.** Escolhas do produto sem norma por
  trás: o suficiente para levantar uma gravação baixa e corrigir uma faixa de
  ouvido, não tanto que o ruído seja levantado ou que o ajuste anule o
  objetivo.
- **A normalização está ativada nas instalações novas.** Atualizar a
  aplicação mantém a definição que a sua configuração já tem, por isso uma
  instalação que a tinha desativada continua desativada até a ativar.
- **O objetivo de loudness do medidor** (Definições → Medidores) é uma
  definição à parte, por predefinição o valor R128 de −23 LUFS. A normalização
  não o usa.

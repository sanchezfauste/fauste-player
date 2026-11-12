# Definições

Abra as **Definições** na barra superior. Feche-as com **Fechar** ou `Esc`.
A maioria das alterações aplica-se de imediato e é guardada automaticamente.

A janela tem um único tamanho (900 × 640, mais pequena num ecrã pequeno),
seja qual for a secção, e a secção desloca-se dentro dela. Cada secção alinha
as suas etiquetas numa só coluna.

As secções Leitores, Medidores, Análise, Loudness, Automatização e Atalhos de teclado têm um botão
**Repor predefinições** no cabeçalho. Pede confirmação e depois repõe apenas
essa secção (os Leitores mantêm o número de leitores e o idioma; os Atalhos
não têm outro botão de reposição). As Saídas de áudio, Listas, Cartucheira,
Biblioteca, MIDI e Remoto não têm nenhum.

## Definições pendentes {#settings-pending}

Todas as definições se aplicam com a aplicação em execução. O sistema de
áudio, a frequência de amostragem, o tamanho do buffer (também o de um
dispositivo próprio), as saídas Main e Cue (leitores e cartucheira), os
dispositivos bit-perfect e as definições de DSD alteram a forma como um
dispositivo de saída é aberto, por isso aplicam-se assim que isso é seguro:

- As definições de um dispositivo aplicam-se quando nada toca nesse
  dispositivo: nenhum leitor encaminhado para ele está a tocar, a fazer fade
  ou a pré-escutar (CUE, também quando retido), e nenhum cartucho toca nele.
  Um leitor em pausa, ou com uma faixa apenas carregada, não retém um
  dispositivo: a sua faixa fica em pausa onde estava.
- A saída de um leitor ou da cartucheira muda quando esse leitor (ou a
  cartucheira) não está a tocar; os outros leitores nos mesmos dispositivos
  não são interrompidos.
- Um sistema de áudio novo aplica-se quando nada toca em lado nenhum.

Uma alteração em espera corre no primeiro momento em que nada do que ela
afeta toca. Pode ser o intervalo entre o fim de uma faixa e o começo da
seguinte: a faixa seguinte começa então mais tarde, o tempo que o dispositivo
demora a reabrir. **Aplicar agora** (abaixo) força antes a alteração de
imediato.

Uma frequência ou um buffer dado a um dispositivo só conta quando muda
aquilo com que o dispositivo abre: dar a um dispositivo o mesmo valor que o
global, ou limpar um valor desses, não altera nada.

Até lá, a barra superior mostra **Definições pendentes**. Passe o rato por
cima para ver o que está à espera e de quê; clique para ver a lista, que
também mostra qualquer alteração que um dispositivo ou uma saída recusou (o
dispositivo mantém as suas definições atuais; a alteração é tentada de novo
quando a alterar ou premir **Aplicar agora**). O rodapé das Definições diz
«Algumas alterações aguardam que as saídas que afetam fiquem livres.» e
oferece **Aplicar agora**; um aviso curto (por exemplo, que uma definição foi
guardada) pode ocupar durante um momento o lugar desse texto.

**Aplicar agora** aplica de imediato todas as alterações de saída em espera.
Quando isso interromperia algo, lista primeiro os dispositivos e o que toca
neles; **Interromper e aplicar** interrompe então por instantes o áudio
nesses dispositivos: o que estava a tocar continua de onde estava depois de
uma breve pausa, e nada que estivesse em pausa ou parado começa.
**Cancelar** (ou `Esc`) continua à espera.

Os limites e a afinação do motor são editados no ficheiro de configuração
com a aplicação fechada (ver
[Dados e cópias de segurança](data-and-backups.md)).

## Saídas de áudio {#audio-outputs}

As alterações nesta secção aplicam-se com a aplicação em execução, assim que
nada do que afetam está a tocar: ver [Definições pendentes](#settings-pending).

![Definições, Saídas de áudio, vista Básico: o seletor, o sistema de áudio, a frequência de amostragem, o tamanho do buffer e as saídas Main e Cue de cada leitor (aqui o sistema silencioso)](../../images/guide/settings-outputs.png)

No topo, **Mostrar** escolhe **Básico** ou **Avançado**. O Básico mostra o
sistema de áudio, a frequência de amostragem, o tamanho do buffer e as
saídas. O Avançado acrescenta, para cada dispositivo que uma saída usa, a
sua própria frequência e buffer, o interruptor bit-perfect e o modo DSD, e
depois as definições de DSD. Mudar de vista só mostra ou oculta linhas: nada
é alterado nem reposto. Quando o Básico oculta uma definição que está em uso,
uma linha di-lo. Quando nenhuma saída usa já um dispositivo, a sua própria
frequência e buffer, o seu interruptor bit-perfect e o seu modo DSD são
esquecidos no próximo arranque da aplicação: se uma saída o voltar a usar
depois disso, parte dos valores globais. Até lá, escolhê-lo de novo (por
exemplo, depois de trocar dois dispositivos) mantém-nos.

| Definição | Significado |
|---|---|
| Sistema de áudio | A última escolha, **Sem saída (silêncio)**, não toca nada: as linhas temporais correm ao ritmo do tempo real sem placa de som (para uma máquina sem ela, ou para ensaiar). Linux: PipeWire (nas versões que o incluem), PulseAudio, JACK ou ALSA. Windows: WASAPI, ASIO (nas versões que o incluem) ou JACK. macOS: Core Audio ou JACK. Os sistemas em falta neste computador, ou sem dispositivo de saída (um servidor JACK que não está em execução), são mostrados como indisponíveis. «Predefinido do sistema» usa o primeiro disponível por essa ordem. |
| Frequência de amostragem | A frequência a que corre cada saída, a menos que um dispositivo tenha a sua própria (Avançado); os ficheiros são convertidos para ela com reamostragem de alta qualidade. Os dispositivos bit-perfect começam à sua frequência e depois seguem os ficheiros. |
| Tamanho do buffer | Frames por bloco de áudio, a menos que um dispositivo tenha o seu próprio; a latência resultante é mostrada por baixo. As saídas através de um servidor de som ou de um misturador do sistema (PulseAudio, o dispositivo `default` do ALSA) pedem pelo menos 50 ms seja qual for este tamanho, ou o que o dispositivo aceitar, para que um computador ocupado não se ouça; a latência di-lo então (`shared_buffer_ms` no ficheiro de configuração) |
| Saídas por leitor | Para cada leitor, e para a coluna **Música** (o [musicalizador](automation.md)), um dispositivo **Main** (no ar) e um dispositivo **Cue** (pré-escuta), cada um com um par de canais. Uma placa de som que oferece vários perfis de saída (o ALSA lista frontal, surround, hardware direto…) mostra cada um como *placa — perfil*; duas entradas que ainda assim se leriam da mesma forma recebem o seu id de dispositivo entre parênteses. As interfaces multicanal podem transportar vários leitores em pares diferentes. Uma saída Cue que partilha uma saída do ar (o Main de qualquer leitor ou o da cartucheira) é permitida: uma nota por baixo diz que a sua pré-escuta se ouve no ar e que pergunta primeiro enquanto algo toca aí. |
| Testar Main / Testar Cue | Toca um tom curto (1 kHz no Main, 440 Hz no Cue, 1,5 s, −18 dBFS) na saída escolhida, para poder verificar as ligações antes de ir para o ar |
| Biblioteca | A saída Cue da biblioteca, com o seu próprio **Testar Cue**: onde se ouve a [pré-escuta](library.md#pre-listening) da janela da biblioteca (num telemóvel, a do separador Biblioteca). Sem ela, o Play da barra de pré-escuta fica esbatido. No Android a sua escolha vazia lê-se *Predefinido do sistema* em vez de *Nenhuma*: a pré-escuta e o **Testar Cue** tocam então na saída predefinida (ver [Android → Biblioteca](android.md#library)). Uma saída da Biblioteca numa saída do ar funciona como o Cue de um leitor: uma nota por baixo diz que a sua pré-escuta se ouve no ar e que pergunta primeiro enquanto algo toca aí. Está nas vistas Básico e Avançado, também num telemóvel |
| Cartucheira | As saídas Main e Cue da cartucheira. O Main assume por predefinição a saída do sistema. Sem um Cue não há pré-escuta de cartuchos; um Cue numa saída do ar funciona como nos leitores. |
| Frequência de amostragem: *dispositivo* (Avançado) | **Global (...)** usa a frequência de amostragem acima; um valor dá a este dispositivo a sua própria frequência. Só são oferecidas as frequências que o dispositivo indica; uma frequência guardada que ele já não indica continua listada com uma nota de que pode não abrir (o dispositivo recorre então à frequência global). Os valores próprios só se aplicam a dispositivos que uma saída nomeia, não à saída predefinida do sistema, a menos que uma o faça. |
| Tamanho do buffer: *dispositivo* (Avançado) | **Global (...)** usa o tamanho do buffer acima (com o mínimo que uma saída partilhada pede, como acima); um valor dá a este dispositivo o seu próprio, usado tal como está, com a latência por baixo. Um dispositivo que não aceita um tamanho de buffer próprio recorre ao global, e também à frequência global quando não aceita uma frequência própria. |
| Bit-perfect: *dispositivo* (Avançado) | Desativado por predefinição. Um modo especial para reprodução audiófila, não para programas misturados: o dispositivo é aberto com acesso exclusivo e segue a frequência de amostragem de cada faixa. Os leitores cuja saída Main está nele nunca fazem crossfade nem misturam, uma mudança de frequência deixa um breve silêncio entre duas faixas, e o seu volume fica fixo em 100 %. O interruptor fica desativado onde o dispositivo não pode dar acesso exclusivo. Ver [O que custa](bit-perfect.md#what-it-costs). |
| DSD: *dispositivo* (Avançado) | **Converter para PCM** (a predefinição), **DoP** ou, no Linux, **DSD nativo**. Todos os dispositivos o mostram; só são oferecidos os modos que o dispositivo pode aceitar, e uma linha por baixo diz porque não são os outros. Ver [DSD](bit-perfect.md#dsd). |
| Quando outra fonte precisa de uma saída DSD (Avançado) | **Continuar a faixa DSD em PCM** (a predefinição), ou **Manter o DSD e silenciar as outras fontes**. Ver [DSD](bit-perfect.md#dsd). |
| Silêncio DSD (Avançado) | Silêncio enviado antes de um fluxo DSD começar, depois de terminar e na passagem para PCM, para que o conversor sincronize sem estalidos; 200 ms por predefinição, de 0 a 2000. |

Uma saída Cue nunca recorre à saída que o Main usa. Um Cue que nomeia um
dispositivo de um sistema de áudio que este computador não tem significa
«sem cue». Um Cue numa saída do ar (o mesmo dispositivo e canais que um Main,
ou sobrepostos) é permitido: uma nota por baixo diz que a sua pré-escuta se
ouve no ar e que pergunta primeiro enquanto algo toca aí. Um leitor sem saída
Cue tem o botão **CUE** esbatido; passar o rato por cima diz para escolher
aqui uma saída Cue. O mesmo vale para o **Pré-escutar no CUE** da
cartucheira.

Se um dispositivo desaparecer durante a reprodução, os leitores mantêm as
suas linhas temporais, e o dispositivo é aberto de novo quando regressar (ver
[Resolução de problemas](troubleshooting.md)).

## Leitores {#players}

![Definições, Leitores: número de leitores, modo predefinido, duração do fade entre faixas, duração do Stop com fade, mistura automática, cue-in e cue-out, aviso de fim de faixa e idioma](../../images/guide/settings-players.png)

| Definição | Predefinição | Significado |
|---|---|---|
| Idioma | Sistema | Idioma da interface |
| Número de leitores | 4 | Colunas no ecrã principal (um leitor no ar não pode ser removido). O número aplica-se quando larga o controlo deslizante; um número escrito na caixa aplica-se ao premir Enter ou ao sair da caixa, e Escape cancela-o |
| Modo predefinido | CONT | O modo em que os leitores arrancam |
| Duração do fade entre faixas | 1000 ms | Quanto tempo a faixa no ar demora a fazer fade out quando prime Play (seguinte) enquanto toca, e no Anterior. Não se aplica a um leitor cuja saída Main é bit-perfect |
| Duração do Stop com fade | 2000 ms | Quanto tempo o **Stop com fade** demora a fazer o fade out da faixa. Aplica-se a todos os leitores, também aos bit-perfect |
| Mistura automática no ponto MIX | Ativada | Sobrepor as faixas no modo contínuo. Não se aplica a um leitor cuja saída Main é bit-perfect |
| Usar cue-in e cue-out | Ativado | Desativado: os leitores tocam cada faixa do início ao fim do ficheiro; os marcadores são mantidos e os cartuchos continuam a usar os seus. As durações e os totais das listas seguem o mesmo intervalo |
| Aviso de fim de faixa | 10 s | Quando a contagem decrescente começa a piscar a vermelho |

Enquanto a saída Main de um leitor é bit-perfect, uma nota por baixo da
**Mistura automática no ponto MIX** diz que esses leitores nunca misturam
nem fazem crossfade, pelo que a duração do fade entre faixas e a mistura
automática não se lhes aplicam. As duas definições continuam editáveis:
aplicam-se aos outros leitores. Ver [O que custa](bit-perfect.md#what-it-costs).

## Medidores {#meters}

![Definições, Medidores, com o medidor de pico digital escolhido](../../images/guide/settings-meters.png)

As alterações aplicam-se de imediato. As Definições só mostram o que o tipo
de medidor escolhido usa: os medidores EBU, DIN e VU têm a escala, a zona
vermelha e o comportamento que a sua norma fixa (só o nível de alinhamento é
definido), e o alinhamento de um medidor K-System é o seu próprio 0. Um valor
que definir é mantido para quando escolher esse tipo de novo.

| Definição | Predefinição | Significado |
|---|---|---|
| Tipo de medidor | Pico digital | Como a barra sobe e desce, e a sua escala, segundo uma norma (ver abaixo) |
| Tempo de subida, Velocidade de descida | 5 ms, 11,8 dB/s | Só para **Personalizado**. O tempo de subida é um tempo de integração: uma rajada de tom dessa duração lê 2 dB abaixo; 0 mostra todos os picos. |
| True peak | Desativado | Só pico digital, personalizado e K-System. Mede entre amostras, com o filtro de sobreamostragem 4× que a ITU-R BS.1770 publica. Mostra os picos que excedem 0 dBFS após a conversão, que um medidor de pico de amostra não vê. Como a norma permite, um clique isolado de uma amostra pode ler até cerca de 0,3 dB abaixo do seu valor de amostra. |
| Base da escala | −60 dBFS | A base da escala digital (pico digital e personalizado). Os outros medidores mostram o intervalo que a sua norma indica. |
| Retenção do pico | 2 s | Só pico digital, personalizado e K-System: durante quanto tempo o nível mais alto fica aceso; 0 desativa-a. Os medidores de programa e o VU não têm retenção. |
| Nível de alinhamento | −18 dBFS | Todos menos o K-System. Marcado na escala (EBU R68). É também onde ficam a marca TEST da EBU, a marca −9 da DIN e o 0 VU. |
| Aviso a partir de | −9 dBFS | Amarelo a partir daqui (máximo permitido EBU), para os medidores de pico digital e personalizado |
| Perigo a partir de | −3 dBFS | Vermelho a partir daqui, para os medidores de pico digital e personalizado. Os outros ficam vermelhos onde a sua escala o faz: o VU a partir de 0 VU, os PPM EBU e DIN a partir do máximo permitido (EBU +9, DIN 0), o K-System a partir de +4. |
| Leitura de loudness | Curto prazo | O loudness por baixo do medidor: desativada, momentânea (últimos 400 ms) ou de curto prazo (últimos 3 s), EBU R128 |
| Objetivo de loudness | −23 LUFS | A leitura fica verde a ±1 LU (EBU R128) |

| Tipo de medidor | Norma | Comportamento |
|---|---|---|
| Pico digital | IEC 60268-18 | Mostra todos os picos de imediato; desce 20 dB em 1,7 s |
| PPM EBU | IEC 60268-10 tipo IIb | Os picos com menos de cerca de 10 ms leem mais baixo (uma rajada de tom de 10 ms lê cerca de 1,6 dB abaixo, uma de 0,5 ms cerca de 18 dB abaixo), dentro das tolerâncias da EBU Tech 3205; desce 24 dB em 2,8 s |
| PPM DIN | IEC 60268-10 tipo I | O mesmo com um tempo de integração de 5 ms; desce 20 dB em 1,5 s |
| VU | IEC 60268-17 | O nível médio, com o movimento da agulha de um medidor VU: 99 % em 300 ms, com uma ligeira ultrapassagem; uma sinusoide lê o seu nível de pico |
| K-20, K-14, K-12 | K-System | Duas secções: o nível médio (RMS, 600 ms) como barra sólida e o pico (desce 26 dB em 3 s) esbatido por cima. O 0 fica 20, 14 ou 12 dB abaixo da escala completa; verde abaixo de 0, âmbar de 0 a +4, vermelho acima. O K-12 serve a radiodifusão, o K-14 e o K-20 programas mais dinâmicos. |
| Personalizado | — | O seu tempo de subida e velocidade de descida |

Cada medidor usa a escala da sua norma, com as suas marcas entre os canais:

| Medidor | Escala |
|---|---|
| Pico digital, personalizado | −60 … 0 dBFS, marcas a cada 10 dB até −40 e a cada 5 dB acima; os 20 dB superiores ocupam metade da altura |
| PPM EBU | −12 … +12 em torno do nível de alinhamento (TEST), a cada 4 dB; os níveis mais baixos ficam na base |
| PPM DIN | −50 … +5, onde 0 fica 9 dB acima do nível de alinhamento (−9 dBFS por predefinição) |
| VU | −20 … +3 VU, 0 VU no nível de alinhamento; a barra move-se em proporção à tensão, como a agulha |
| K-System | de +20, +14 ou +12 (0 dBFS) até −60; uniforme em dB até −24 |

## Análise {#analysis}

![Definições, Análise: os limiares dos marcadores automáticos](../../images/guide/settings-analysis.png)

Os limiares descritos em [Marcadores e mistura](markers-and-mixing.md); cada
linha tem por baixo do nome uma dica de uma linha a dizer o que o valor faz.
**Voltar a analisar todas as faixas** executa a análise de novo para toda a
biblioteca, lendo todos os ficheiros outra vez; os marcadores manuais são
mantidos. Enquanto corre, o botão fica esbatido e mostra até onde chegou
(**A analisar… 120 de 340**), por isso premi-lo de novo não recomeça; a dica
do **A analisar N faixas** da barra de estado dá o mesmo progresso. Alterar
aqui um limiar também analisa de novo a biblioteca, e reinicia uma execução
já em curso para que nenhuma faixa fique com marcadores dos valores antigos.

Após uma atualização cuja análise mudou, as faixas analisadas pela versão
anterior mantêm os seus marcadores e formas de onda, que continuam a
funcionar. No arranque, o Fauste Player diz quantas são e oferece **Analisar
agora** ou **Mais tarde**; **Analisar faixas desatualizadas (N)** aqui faz o
mesmo em qualquer altura. As faixas nos leitores são postas em dia de
qualquer forma, à medida que são mostradas, e o mesmo acontece com as faixas
de cartuchos que não têm formato registado (um cartucho só toca bit-perfect
quando o seu formato é conhecido). As faixas cujo ficheiro está em falta não
são contadas até o ficheiro voltar.

A análise também mede o loudness de cada ficheiro para a
[normalização do loudness](loudness.md). Os ficheiros analisados antes de o
loudness ser medido são medidos em segundo plano, quando a análise não tem
mais nada a fazer, com a normalização ativada ou desativada; os seus
marcadores não são tocados. **Voltar a analisar** (no menu da lista) e
**Voltar a analisar todas as faixas** medem de novo.

**Procurar ficheiros alterados** verifica todas as faixas da biblioteca à
procura de um ficheiro que outro programa tenha alterado enquanto o Fauste
Player corria, e analisa de novo os alterados; um aviso diz quantos
encontrou. O botão fica esbatido enquanto uma verificação decorre. Ver
[Quando um ficheiro muda](library.md#when-a-file-changes).

## Loudness {#loudness}

![Definições, Loudness: a normalização ativada, com o objetivo, o teto de true peak, o ganho máximo e o ajuste máximo nas predefinições](../../images/guide/settings-loudness.png)

Toca cada faixa e cada cartucho com o mesmo loudness, com um ganho fixo por
faixa. A [normalização do loudness](loudness.md) explica como o ganho é
calculado, o indicador no cabeçalho do leitor e as definições por faixa.

| Definição | Predefinição | Intervalo | Significado |
|---|---|---|---|
| Normalizar o loudness | Ativada | | Normalização ativada ou desativada |
| Objetivo | −16 LUFS | −36 … −6 LUFS, passos de 0,5 | O loudness integrado a que cada faixa é levada |
| Teto de true peak | −1 dBTP | −12 … 0 dBTP, passos de 0,1 | Um aumento nunca leva o true peak da faixa acima disto. Os cortes não são limitados |
| Ganho máximo | 12 dB | 0 … 24 dB | O maior ganho que uma faixa baixa pode receber. Os cortes não são limitados |
| Ajuste máximo | 12 dB | 0 … 24 dB | Até onde pode ir o ajuste manual de cada faixa, para cima ou para baixo |

As alterações aplicam-se às faixas carregadas depois: o que está a tocar
mantém o seu ganho, e a faixa seguinte à espera em cada leitor é preparada de
novo com o novo. Uma nota por baixo das definições diz quantas faixas ainda
esperam pela sua medição; tocam sem alterações até estar feita. Um leitor ou
cartucho numa saída bit-perfect nunca é normalizado. **Repor predefinições**
repõe apenas esta secção (normalização ativada). A normalização está ativada
nas instalações novas; uma configuração existente mantém a sua definição
atual (toda a configuração guardada já armazena o valor).

## Listas {#playlists}

![Definições, Listas: a pasta de música, as listas e as colunas da tabela](../../images/guide/settings-playlists.png)

- **Pasta de música:** onde começam os diálogos de ficheiros.
- **Nova lista**, **mudar o nome** (edite o nome e prima Enter; Esc cancela)
  e **eliminar** (ícone do caixote do lixo).
- **Importar M3U / PLS…** cria uma lista nova a partir de um ficheiro de
  lista. **M3U** em cada linha exporta-a como M3U8. Ver
  [Listas](playlists.md).
- **Colunas da tabela:** que colunas as tabelas de faixas mostram e por que
  ordem, para cada leitor: uma caixa de seleção por coluna (o Título e a Dur.
  não podem ser desativados), setas para cima e para baixo para as mostradas
  e **Colunas predefinidas**. Ver [Listas](playlists.md).

**Idioma:** uma lista pendente: **Sistema** (seguir o sistema operativo),
depois todos os idiomas em que a interface está disponível, cada um com o seu
próprio nome (o inglês primeiro, depois por ordem alfabética: por exemplo,
Español). A interface muda de imediato. Um idioma do sistema sem tradução
própria usa o mais próximo (o francês do Canadá usa o francês, o português
do Brasil usa o português) e, caso contrário, o inglês. Um idioma no
ficheiro de definições que a interface não tem aparece como **Sistema** e
segue o sistema operativo.

A interface em todos os idiomas, o inglês e o espanhol incluídos, foi
produzida com IA e pode conter erros. A janela **Acerca do Fauste Player**
traz um aviso geral que os abrange a todos. As correções são bem-vindas como
issues ou pull requests.

## Cartucheira {#cartwall}

![Definições, Cartucheira: as páginas, a grelha e o editor do cartucho selecionado](../../images/guide/settings-cartwall.png)

Páginas, tamanho da grelha, o editor de cartuchos e a importação e
exportação de páginas de cartuchos. Ver [Cartucheira](cartwall.md).

## Biblioteca {#library}

Criar, renomear, recolorir, mudar o tipo de, mover (arrastando) e eliminar as
categorias que agrupam as suas faixas, vigiar uma pasta com uma categoria e
remover as suas faixas em falta, sem abrir a janela da biblioteca. Ver
[Biblioteca](library.md#categories) e
[Pastas vigiadas](library.md#watched-folders).

**Editor externo** é o editor de áudio que *Abrir no editor externo* inicia
com o ficheiro de uma faixa: o caminho do programa (ou *Nenhum*), **Escolher…**
para o escolher e **Limpar** para o remover. No macOS pode escolher-se um
pacote de aplicação (`.app`). Se o programa não puder ser iniciado, um aviso
di-lo. A linha não é mostrada no Android, nem nas versões Flatpak e snap. Ver
[Quando um ficheiro muda](library.md#when-a-file-changes).

## Automatização {#automation}

As rotações e as regras do [musicalizador](automation.md) (não no Android).

![Definições, Automatização: as rotações Music (a predefinida) e Hits, a Music selecionada com o seu único passo sobre toda a música, e as regras do musicalizador](../../images/guide/settings-automation.png)

- **Rotações:** a lista, com a predefinida marcada. **Nova rotação**
  adiciona uma que reproduz toda a música, **Tornar predefinida** torna a
  selecionada a predefinida, **Eliminar rotação** remove-a (a última não pode
  ser eliminada).
- **A rotação selecionada:** o seu **Nome**, depois uma linha por **passo**:
  de onde vêm as suas faixas (uma categoria com as suas subcategorias, ou
  *Tudo* de um tipo), quantas reproduz de cada vez (**No mínimo** 0–99,
  **No máximo** 1–99; um mínimo de 0 deixa o passo ficar de fora em algumas
  voltas), **Subir**, **Descer** e **Remover**. **Adicionar passo**
  adiciona um no fim. Um passo cuja categoria foi eliminada di-lo; o
  musicalizador salta-o.
- **Musicalizador:** quanto tempo antes de a mesma faixa poder tocar de novo
  (3 horas, ou quando metade das outras faixas do seu passo tiverem sido
  emitidas, o que ocorrer primeiro; 0 desliga qualquer das duas), o mesmo
  artista (30 minutos), a **margem à frente** (30 minutos de música em fila)
  e as **faixas tocadas que se mantêm** na sua lista (50).

**Repor predefinições** repõe as regras; as rotações ficam.

## Atalhos de teclado {#keyboard-shortcuts}

![Definições, Atalhos de teclado: cada ação do leitor com a sua tecla, e Desassociar ao lado das associadas](../../images/guide/settings-shortcuts.png)

Ver [Teclado](keyboard.md).

## MIDI {#midi}

![Definições, MIDI, com o controlo MIDI desativado](../../images/guide/settings-midi.png)

Ative as superfícies de controlo MIDI, veja as portas de entrada (**Voltar a ligar as portas** abre-as de novo,
como voltar a ligar os controladores) e aprenda um controlo para cada ação do
leitor. Ver
[Superfícies de controlo MIDI](midi.md).

## Remoto {#remote}

![Definições, Remoto, com a API HTTP à escuta neste computador](../../images/guide/settings-remote.png)

Controlo remoto pela rede, para páginas web, aplicações de telemóvel,
automação e superfícies de controlo. Ver [Controlo remoto](remote-control.md).

- **Permitir o controlo remoto por HTTP**, o seu **endereço** e **porta**, e
  uma linha que diz se está à escuta.
- **Token**, obrigatório fora deste computador. **Gerar** cria um aleatório,
  **Mostrar** revela-o e **Copiar** põe-no na área de transferência. Aparece
  um aviso quando o endereço chega além deste computador e não há token.
- **Páginas web autorizadas a usar a API**: uma origem por linha.
- **Permitir o controlo por OSC**, o seu **endereço** e **porta**, e os
  **emissores autorizados** (endereços ou sub-redes, um por linha).
- **Publicar os tempos a cada**: com que frequência os tempos decorrido e
  restante são enviados enquanto algo toca.

Os campos de texto e os números aplicam-se quando os deixa, o que inclui
abrir outra secção ou fechar as Definições; Esc cancela o que estava a
escrever. Um valor inválido é corrigido, e o campo mostra o que foi mantido.

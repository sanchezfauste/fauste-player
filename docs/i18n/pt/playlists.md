# Listas

## Separadores {#tabs}

Cada leitor tem uma fila de separadores, um por lista. Todos os leitores veem
as mesmas listas; cada leitor escolhe qual mostra. Um ponto num separador
mostra onde estão as faixas do leitor: **vermelho** para a faixa no ar,
**verde** para a seguinte.

Os separadores partilham a largura do leitor. Um nome que não cabe termina em
«…»; passe o rato sobre o separador para o ler por inteiro. Com muitas
listas, os separadores deixam de encolher numa largura mínima, aparecem
setas nas pontas da fila e a roda do rato sobre os separadores desloca-os. O
separador que escolhe, e o que um leitor mostra, é trazido para a vista.

Mudar de separador nunca altera o que está no ar nem o que é a seguinte.
Quando uma faixa termina, o leitor continua na lista que contém essa faixa.

As listas são criadas, renomeadas e eliminadas nas [Definições](settings.md).
A última lista, e uma lista com uma faixa no ar, não podem ser eliminadas.

## A tabela de faixas {#the-track-table}

![Uma lista: as faixas tocadas esbatidas, a faixa no ar a vermelho, a faixa seguinte a verde e o rodapé com o tempo restante](../../images/guide/playlist.png)

| Coluna | Conteúdo |
|---|---|
| `#` | Posição, com zeros à esquerda; um ícone substitui-a nas faixas atual e seguinte |
| Título | Das etiquetas, ou do nome do ficheiro (`Artista - Título.mp3` é separado). Os ícones de repetir e de parar após a faixa de uma faixa ficam antes dele |
| Artista | Das etiquetas; «Artista desconhecido» quando não há nenhum |
| Álbum | Das etiquetas |
| Data | A data de gravação tal como o ficheiro a guarda (`2019`, `2019-05` ou `2019-05-14`, com uma hora se houver) |
| Género | Das etiquetas |
| Dur. | Duração de reprodução, do cue-in ao cue-out (o ficheiro inteiro com **Usar cue-in e cue-out** desativado) |
| Intro | Quanto dura a intro, desde onde a faixa começa a tocar até ao seu marcador de intro; vazio quando a faixa não tem marcador de intro |
| Ficheiro | O nome do ficheiro, com a sua extensão |

Uma instalação nova mostra `#`, Título, Artista e Dur. As outras colunas são
opcionais; veja **Escolher as colunas** abaixo. Uma faixa que não tem um
valor mostra uma célula vazia, exceto o Artista, que mostra «Artista
desconhecido».

As colunas preenchem a tabela e mantêm as suas proporções quando a janela é
redimensionada; as colunas de texto recebem mais espaço. Arraste os
separadores do cabeçalho para alterar as proporções: as colunas à direita do
separador seguem o ponteiro a cada frame (partilham o que resta na
proporção da sua largura), as da esquerda ficam, e as larguras são guardadas
quando larga. Nenhuma coluna fica mais estreita do que o seu mínimo. As
larguras são recordadas por leitor; uma coluna que mostrar mais tarde começa
com a sua largura predefinida e as outras mantêm as suas proporções.

### Escolher as colunas {#choosing-the-columns}

O Título e a Dur. são sempre mostrados. Todas as outras colunas podem ser
mostradas ou ocultadas, e qualquer coluna, incluindo essas duas, pode ser
movida. A lista é a mesma para todos os leitores e listas, e é guardada em
`config.json` como `ui.table_columns`. Três formas de a alterar:

- **Definições → Listas → Colunas da tabela:** assinale as colunas a
  mostrar; as setas movem uma coluna mostrada para cima ou para baixo (leem-se
  da esquerda para a direita nas tabelas). **Colunas predefinidas** volta a
  `#`, Título, Artista e Dur.
- **Clique com o botão direito num cabeçalho:** um menu com uma caixa de
  seleção para cada coluna opcional. Uma coluna que mostrar aparece na
  extremidade direita; arraste-a a partir daí.
- **Arraste um cabeçalho** para cima de outro: largue-o na metade esquerda de
  um cabeçalho para pôr a coluna antes dele, na metade direita para a pôr
  depois. Largar noutro sítio qualquer não faz nada.

Um nome em `ui.table_columns` que esta versão não conhece é ignorado, e um
Título ou Dur. em falta é acrescentado de novo.

Quando a aplicação abre, cada tabela desloca-se de modo a que a faixa
seguinte do seu leitor fique no meio da tabela (o mais perto que as pontas da
lista permitam). Isto acontece uma só vez, no arranque, e apenas quando a
faixa seguinte está na lista que a tabela mostra.

Quando um leitor passa para outra faixa, a sua tabela mostra a lista dessa
faixa e desloca a sua linha para o topo, a menos que tenha usado a tabela nos
últimos 10 segundos (deslocou-a, arrastou uma faixa, abriu o menu de uma
faixa ou clicou num separador): então espera até a deixar em paz durante
esse tempo. O tempo é `ui.follow_current_grace_secs` em `config.json`; 0
desativa o seguimento.

Os leitores são independentes: vários leitores podem mostrar a mesma lista,
cada um com a sua própria faixa seguinte, as suas próprias marcas de tocada
e os seus próprios tempos no rodapé. Tocar, parar ou saltar num leitor nunca
move a seguinte de outro leitor. A mesma faixa pode até estar no ar em dois
leitores ao mesmo tempo. Editar a lista (adicionar, mover ou remover
entradas) ou um ficheiro tornar-se ilegível pode, ainda assim, alterar a
seguinte de qualquer leitor que a mostre.

Cores das linhas:

| Linha | Significado |
|---|---|
| **Vermelho**, com um ícone de altifalante (ou de pausa) | No ar neste leitor. Também pode mostrar a seta verde: a faixa no ar é também a seguinte, por isso toca mais uma vez |
| **P2** a vermelho (ou outro número) na coluna do número | No ar nesse leitor |
| **Verde**, com uma seta | A faixa seguinte deste leitor |
| Esbatida | Já tocada neste leitor |
| Ficheiro com uma cruz / ícone de aviso | Ficheiro em falta / ilegível (é saltado); passe o rato sobre a linha: a janela começa com o motivo, a âmbar, e depois os campos habituais. Um ficheiro em falta é procurado de novo a cada 30 s (`tuning.missing_recheck_ms`). |
| Setas de recarregar à direita do título | Analisada por uma versão anterior; continua a tocar com essa análise. **Definições → Análise → Analisar faixas desatualizadas** põe-na em dia (as faixas num leitor são atualizadas de qualquer forma) |
| Ampulheta à direita do título | A faixa está à espera da sua análise (passe o rato sobre a ampulheta: *Análise pendente*). Toca na mesma, e a ampulheta desaparece quando a análise termina |
| Violeta | Selecionada |

**Dica da faixa.** Passe o rato sobre uma linha durante um momento para ver o
seu título, artista, álbum, data, género, duração, formato (tipo, frequência
de amostragem e profundidade de bits, quando conhecidos) e o caminho do seu
ficheiro. Um campo que o ficheiro não tem é omitido. A janela é a única
informação ao passar o rato sobre uma linha, e nunca se move enquanto é
mostrada. Para um ficheiro em falta ou ilegível, começa com o motivo.

## Rato {#mouse}

- **Clique** seleciona uma faixa. **Duplo clique** torna-a a faixa seguinte
  deste leitor. Na faixa no ar, faz com que toque mais uma vez quando a
  passagem atual terminar.
- **Clique com o botão direito** abre o menu de contexto:

![O menu de contexto de uma faixa](../../images/guide/track-menu.png)

| Item | Ação |
|---|---|
| Reproduzir agora | Iniciar esta faixa de imediato (misturando se o leitor estiver no ar) |
| Definir como seguinte | O mesmo que o duplo clique. Na faixa no ar, toca mais uma vez, desde o início, quando a passagem atual terminar (misturando como o Repetir, sem interrupção), e depois o leitor continua. Atua uma só vez. O Stop no fim, o modo SINGLE e uma marca de Stop no fim continuam a terminar primeiro o leitor. Enquanto um CUE decorre, passa para a nova seguinte |
| Pré-escutar no CUE | Tocá-la na saída CUE (abre a janela de CUE). Esbatido quando o leitor não tem uma saída Cue à parte da sua saída Main |
| Editar etiquetas… | Abrir o editor de etiquetas desta faixa. **Guardar** escreve as alterações no ficheiro de áudio; **Cancelar** (ou Esc, quando nenhuma gravação está em curso) fecha sem escrever. O item fica esbatido, com o motivo ao passar o rato, enquanto a faixa está no ar, em CUE ou num cartucho a tocar, enquanto as suas etiquetas ainda não foram lidas, quando o ficheiro está em falta e para formatos cujas etiquetas não podem ser escritas (por exemplo, DSD) |
| Voltar a analisar | Analisar esta faixa de novo agora, seja qual for o seu estado. Um ficheiro corrigido que estava ilegível também é recolhido por si só (ver [Resolução de problemas](troubleshooting.md)). Os marcadores manuais são mantidos |
| Adicionar faixas abaixo… | Escolher ficheiros para inserir depois desta faixa |
| Duplicar | Inserir abaixo uma cópia por tocar (com as suas marcas de repetir e de parar após a faixa) |
| Repetir esta faixa | Assinale para a tocar vezes sem conta, sem interrupção, até premir Play (seguinte), Anterior, Stop ou Stop com fade, ou ativar Stop no fim. A Pausa mantém a repetição. Um ícone de repetir aparece antes do título |
| Parar após esta faixa | Assinale para parar o leitor quando esta faixa termina, sempre que toca (em qualquer modo). Ao contrário do botão **Stop no fim** do leitor, a marca fica com a faixa e é guardada com a lista. O ícone de parar após a faixa aparece antes do título. Prevalece sobre o Repetir |
| Mover para ▸ | Movê-la para o fim de outra lista |
| Remover da lista | Removê-la; não é possível enquanto está no ar |

## Editar etiquetas {#editing-tags}

**Editar etiquetas…** abre uma janela para uma faixa. Enquanto está aberta,
nenhum atalho de teclado atua e os ficheiros largados na janela da aplicação
são ignorados.

![O editor de etiquetas de um ficheiro FLAC, com a sua capa, título, artista, álbum, data e género](../../images/guide/tag-editor.png)

- **O que vê.** O editor lê o ficheiro quando abre (entretanto mostra «A ler
  etiquetas…»). Sempre mostrados: título, artista, álbum, artista do álbum,
  data, número da faixa e total, número do disco e total, género, compositor
  e comentário. Mostrados quando o ficheiro os tem: subtítulo, agrupamento,
  BPM, tonalidade inicial, ambiente, ISRC, editora, número de catálogo,
  copyright, artista original, álbum original, data de lançamento original,
  letrista, maestro, remisturador, arranjador, intérprete, idioma, codificado
  por, letra, título para ordenar, artista para ordenar, álbum para ordenar,
  artista do álbum para ordenar, compositor para ordenar e site do artista.
- **Adicionar campo.** O menu por baixo dos campos lista os outros campos.
  Oferece apenas o que o formato de etiquetas do ficheiro pode guardar (um
  WAV com RIFF INFO, um AIFF ou uma etiqueta ID3v1 antiga guardam menos
  campos do que ID3v2, FLAC ou MP4), e fica esbatido quando não há mais nada
  a adicionar. Um dos campos sempre mostrados que o formato não pode guardar
  aparece a cinzento com uma nota. Limpar um campo remove-o do ficheiro; um
  campo adicionado que fique vazio não é escrito.
- **Vários valores.** Os campos que podem guardar vários valores (artista,
  artista do álbum, género, compositor, ambiente e os créditos como letrista,
  maestro, remisturador, arranjador e intérprete, e idioma) mostram um valor
  por linha; **Guardar** escreve um valor por linha à maneira do próprio
  formato. O comentário e a letra são texto livre em várias linhas.
- **Verificações.** A data e a data de lançamento original são ISO 8601
  (`2019`, `2019-05` ou `2019-05-14`, opcionalmente com uma hora); o número
  da faixa e do disco, os seus totais e o BPM são números inteiros, e um total
  precisa do seu número. Um campo com um valor inválido fica assinalado e o
  **Guardar** continua desativado. Um valor que o ficheiro já tinha e que não
  tocou é mantido tal como está.
- **Campos demasiado longos.** Um campo cujo texto é mais longo do que
  `limits.max_tag_chars`, ou que guarda mais valores do que
  `limits.max_tag_values`, é mostrado só de leitura com a nota «Demasiado
  longo para editar aqui; mantém-se tal como está no ficheiro». Nunca é
  escrito de volta, por isso uma gravação não o pode cortar.
- **O que é mantido.** Tudo o que o editor não mostra (outras chaves
  standard, chaves personalizadas, imagens que não sejam a capa frontal,
  frames binárias) fica no ficheiro com os mesmos valores. O editor diz
  quantas etiquetas desse tipo são mantidas (e «outras» quando o formato tem
  frames que não podem ser contadas). Guardar volta a codificar os itens que
  o editor mapeia, por isso um item mantido pode diferir nos seus bytes
  (codificação de texto, ordem das frames), mas não no seu valor.
- **A capa.** O editor mostra a capa frontal, ou a primeira imagem do
  ficheiro quando não há capa frontal, como miniatura.
  - **Alterar…** abre um diálogo de ficheiros para uma imagem JPEG ou PNG (no
    máximo `limits.max_cover_bytes`, e tem de descodificar). Se não
    descodificar, o editor diz porquê e nada muda.
  - **Remover** limpa a capa frontal. Fica desativado quando o ficheiro não
    tem capa frontal: uma imagem mostrada apenas porque não há capa frontal
    serve só para visualização e é mantida tal como está.
  - Uma capa que está no ficheiro mas não pode ser mostrada (uma imagem que
    não descodifica, ou um GIF, BMP ou WebP) é anunciada com «Esta capa não
    pode ser mostrada; mantém-se tal como está». **Alterar…** e **Remover**
    continuam a funcionar.
  - A alteração é escrita por **Guardar** e descartada por **Cancelar**. As
    contracapas e todas as outras imagens nunca são tocadas. Um formato sem
    lugar para imagens (WAV com RIFF INFO, AIFF, ID3v1) mostra a área
    desativada. Após uma gravação, a capa do leitor mostra a nova capa.
- **Como funciona uma gravação.** O ficheiro é copiado ao lado do original, a
  cópia recebe as etiquetas, é sincronizada e substitui o original, por isso
  uma falha deixa o ficheiro como estava. O motivo aparece no editor, que
  fica aberto para tentar de novo, e na barra de estado. Só os campos que
  alterou são escritos. Após uma gravação, a tabela mostra de imediato as
  novas etiquetas, e os marcadores e a forma de onda são mantidos. Se o
  ficheiro não manteve um campo que alterou, a barra de estado nomeia-o.
- **Após uma atualização.** As faixas de uma versão anterior veem a sua data,
  género e outras etiquetas preenchidos discretamente em segundo plano (sem
  análise completa).

## Arrastar e largar {#drag-and-drop}

- Arraste uma faixa dentro da lista para a reordenar. Uma linha violeta
  mostra onde vai aterrar: fica no limite de linha mais próximo do ponteiro,
  e só na lista sob o ponteiro. Largar sobre o cabeçalho, a borda de uma
  coluna, a barra de deslocamento ou uma janela que cubra a lista (a janela de
  CUE) não larga nada.
- Enquanto arrasta uma faixa, mantenha o ponteiro perto da borda superior ou
  inferior de uma lista para a deslocar: quanto mais perto da borda, mais
  depressa vai, e para nas pontas da lista ou quando se afasta da borda. A
  roda do rato também desloca a lista durante o arrastamento. A linha
  violeta continua a seguir o ponteiro à medida que a lista se move. Arrastar
  ficheiros do gestor de ficheiros sobre uma lista desloca-a da mesma forma
  onde o sistema comunica a posição do ponteiro, depois de mover o ponteiro
  sobre a lista.
- Arraste-a para a lista de outro leitor para a mover para lá.
- Arraste-a para um separador para a acrescentar a essa lista.
- Largue ficheiros ou pastas do gestor de ficheiros sobre uma lista para os
  inserir na posição onde larga. Sobre o cabeçalho, a borda de uma coluna, a
  barra de deslocamento ou uma janela que cubra a lista, nada é inserido. Se
  o sistema não comunicar a posição, vão para o fim da lista mostrada.

## Rodapé {#footer}

**+ Adicionar** abre um diálogo de ficheiros, a começar na pasta de música
definida nas Definições. **Repor tocadas** (o ícone de seta ao lado) limpa a
marca esbatida de «já tocada» de todas as faixas da lista, para todos os
leitores, depois de perguntar «Limpar a marca de tocada de todas as faixas
desta lista?» (**Cancelar**, Esc ou um clique fora mantêm as marcas). A
faixa que está no ar mantém o seu estado e é marcada quando o leitor a
deixa. O botão fica esbatido quando não há nada a limpar. O rodapé mostra
também o número de faixas, o tempo que resta na lista e a sua duração total.

## Ficheiros de lista {#playlist-files}

- **Importar:** Definições → Listas → **Importar M3U / PLS…**, ou largue um
  ficheiro `.m3u`, `.m3u8` ou `.pls` na janela. Torna-se uma lista nova com
  o nome do ficheiro.
  - Os caminhos relativos são resolvidos em relação à pasta do ficheiro de
    lista.
  - Os endereços `file://` são compreendidos.
  - Os ficheiros que não são encontrados são adicionados na mesma, marcados
    como indisponíveis.
  - Os streams da Internet são ignorados; uma mensagem diz quantos.
- **Exportar:** o botão **M3U** de cada lista nas Definições guarda-a como um
  ficheiro M3U8 com títulos, durações e caminhos completos.

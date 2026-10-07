# Definições

Abra as **Definições** na barra superior. Feche-as com **Fechar** ou `Esc`.
A maioria das alterações aplica-se de imediato e é guardada automaticamente.

A janela tem um único tamanho (900 × 640, mais pequena num ecrã pequeno),
seja qual for a secção, e a secção desloca-se dentro dela. Cada secção alinha
as suas etiquetas numa só coluna.

As secções Leitores, Medidores, Análise e Atalhos de teclado têm um botão
**Repor predefinições** no cabeçalho. Pede confirmação e depois repõe apenas
essa secção (os Leitores mantêm o número de leitores e o idioma; os Atalhos
não têm outro botão de reposição). As Saídas de áudio, Listas, Cartucheira,
MIDI e Remoto não têm nenhum.

## Reinício pendente {#restart-pending}

Algumas alterações só têm efeito quando a aplicação arranca de novo: o
sistema de áudio, a frequência de amostragem, o tamanho do buffer (também o
de um dispositivo próprio), as saídas Main e Cue (leitores e cartucheira), os
dispositivos bit-perfect e as definições de DSD. O número de leitores não é
uma delas: aplica-se de imediato.

Uma frequência ou um buffer dado a um dispositivo só conta quando muda
aquilo com que o dispositivo abre: dar a um dispositivo o mesmo valor que o
global, ou limpar um valor desses, não fica pendente.

Os limites e a afinação do motor também se aplicam no arranque seguinte, mas
são editados no ficheiro de configuração com a aplicação fechada (ver
[Dados e cópias de segurança](data-and-backups.md)), por isso nunca aparecem
como pendentes.

Enquanto uma destas está à espera, o rodapé das Definições diz «Algumas
alterações só têm efeito após reiniciar.» e oferece **Reiniciar agora**, e a
barra superior mostra uma etiqueta **Reinício pendente**. Passe o rato sobre
a etiqueta para ver o que está à espera. Um aviso curto (por exemplo, que uma
definição foi guardada) pode ocupar durante um momento o lugar do texto do
rodapé; o **Reiniciar agora** mantém-se. Ambos fazem o mesmo:

- Quando nada está no ar, **Reiniciar agora** (ou a etiqueta) reinicia de
  imediato.
- Quando algo está no ar, aparece a janela que lista o que está a soar, com
  **Parar e reiniciar** ou **Cancelar**.

A sessão é guardada primeiro e o áudio e o controlo MIDI param; depois a
aplicação arranca de novo com a mesma pasta de dados (`FAUSTE_HOME`), e nada
vai para o ar por si só a seguir. Se a aplicação não conseguir arrancar de
novo (num Flatpak, também quando a nova não arranca a tempo), di-lo; inicie-a
a partir do menu de aplicações.

## Saídas de áudio {#audio-outputs}

As alterações nesta secção esperam por um reinício: ver
[Reinício pendente](#restart-pending).

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
| Tamanho do buffer | Frames por bloco de áudio, a menos que um dispositivo tenha o seu próprio; a latência resultante é mostrada por baixo |
| Saídas por leitor | Para cada leitor, um dispositivo **Main** (no ar) e um dispositivo **Cue** (pré-escuta), cada um com um par de canais. Uma placa de som que oferece vários perfis de saída (o ALSA lista frontal, surround, hardware direto…) mostra cada um como *placa — perfil*; duas entradas que ainda assim se leriam da mesma forma recebem o seu id de dispositivo entre parênteses. As interfaces multicanal podem transportar vários leitores em pares diferentes. |
| Testar Main / Testar Cue | Toca um tom curto (1 kHz no Main, 440 Hz no Cue, 1,5 s, −18 dBFS) na saída escolhida, para poder verificar as ligações antes de ir para o ar |
| Cartucheira | As saídas Main e Cue da cartucheira. O Main assume por predefinição a saída do sistema. Sem um Cue não há pré-escuta de cartuchos. |
| Frequência de amostragem: *dispositivo* (Avançado) | **Global (...)** usa a frequência de amostragem acima; um valor dá a este dispositivo a sua própria frequência. Só são oferecidas as frequências que o dispositivo indica; uma frequência guardada que ele já não indica continua listada com uma nota de que pode não abrir (o dispositivo recorre então à frequência global). Os valores próprios só se aplicam a dispositivos que uma saída nomeia, não à saída predefinida do sistema, a menos que uma o faça. |
| Tamanho do buffer: *dispositivo* (Avançado) | **Global (...)** usa o tamanho do buffer acima; um valor dá a este dispositivo o seu próprio, com a latência por baixo. Um dispositivo que não aceita um tamanho de buffer próprio recorre ao global, e também à frequência global quando não aceita uma frequência própria. |
| Bit-perfect: *dispositivo* (Avançado) | Um dispositivo bit-perfect é aberto com acesso exclusivo e segue a frequência de amostragem de cada ficheiro enquanto nada toca nele. O interruptor fica desativado onde o dispositivo não pode dar acesso exclusivo. Ver [Saída bit-perfect](bit-perfect.md). |
| DSD: *dispositivo* (Avançado) | **Converter para PCM** (a predefinição), **DoP** ou, no Linux, **DSD nativo**. Todos os dispositivos o mostram; só são oferecidos os modos que o dispositivo pode aceitar, e uma linha por baixo diz porque não são os outros. Ver [DSD](bit-perfect.md#dsd). |
| Quando outra fonte precisa de uma saída DSD (Avançado) | **Continuar a faixa DSD em PCM** (a predefinição), ou **Manter o DSD e silenciar as outras fontes**. Ver [DSD](bit-perfect.md#dsd). |
| Silêncio DSD (Avançado) | Silêncio enviado antes de um fluxo DSD começar, depois de terminar e na passagem para PCM, para que o conversor sincronize sem estalidos; 200 ms por predefinição, de 0 a 2000. |

Uma saída Cue nunca recorre à saída que o Main usa, para que a pré-escuta
nunca vá para o ar. Um Cue que nomeia um dispositivo de um sistema de áudio
que este computador não tem, ou a mesma saída (dispositivo e canais) que o
Main, significa «sem cue». Quando uma saída Cue é igual à sua saída Main,
um aviso por baixo di-lo. Um leitor sem saída Cue, ou com o Cue na sua saída
Main, tem o botão **CUE** esbatido; passar o rato por cima diz para escolher
aqui uma saída Cue. O mesmo vale para o **Pré-escutar no CUE** da
cartucheira.

Se um dispositivo desaparecer durante a reprodução, os leitores mantêm as
suas linhas temporais, e o dispositivo é aberto de novo quando regressar (ver
[Resolução de problemas](troubleshooting.md)).

## Leitores {#players}

![Definições, Leitores: número de leitores, modo predefinido, duração do fade, mistura automática, cue-in e cue-out, aviso de fim de faixa e idioma](../../images/guide/settings-players.png)

| Definição | Predefinição | Significado |
|---|---|---|
| Idioma | Sistema | Idioma da interface |
| Número de leitores | 4 | Colunas no ecrã principal (um leitor no ar não pode ser removido) |
| Modo predefinido | CONT | O modo em que os leitores arrancam |
| Duração do fade | 1000 ms | Usada pelo Play com uma faixa no ar e pelo Stop com fade |
| Mistura automática no ponto MIX | Ativada | Sobrepor as faixas no modo contínuo |
| Usar cue-in e cue-out | Ativado | Desativado: os leitores tocam cada faixa do início ao fim do ficheiro; os marcadores são mantidos e os cartuchos continuam a usar os seus. As durações e os totais das listas seguem o mesmo intervalo |
| Aviso de fim de faixa | 10 s | Quando a contagem decrescente começa a piscar a vermelho |

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

Os limiares descritos em [Marcadores e mistura](markers-and-mixing.md).
**Voltar a analisar todas as faixas** executa a análise de novo para toda a
biblioteca; os marcadores manuais são mantidos.

Após uma atualização cuja análise mudou, as faixas analisadas pela versão
anterior mantêm os seus marcadores e formas de onda, que continuam a
funcionar. No arranque, o Fauste Player diz quantas são e oferece **Analisar
agora** ou **Mais tarde**; **Analisar faixas desatualizadas (N)** aqui faz o
mesmo em qualquer altura. As faixas nos leitores são postas em dia de
qualquer forma, à medida que são mostradas, e o mesmo acontece com as faixas
de cartuchos que não têm formato registado (um cartucho só toca bit-perfect
quando o seu formato é conhecido). As faixas cujo ficheiro está em falta não
são contadas até o ficheiro voltar.

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

O inglês e o espanhol são escritos à mão. As outras traduções foram geradas
com IA e podem conter erros; quando uma delas está em uso, a janela
**Acerca do Fauste Player** di-lo. As correções de falantes nativos são
bem-vindas como issues ou pull requests.

## Cartucheira {#cartwall}

![Definições, Cartucheira: as páginas, a grelha e o editor do cartucho selecionado](../../images/guide/settings-cartwall.png)

Páginas, tamanho da grelha, o editor de cartuchos e a importação e
exportação de páginas de cartuchos. Ver [Cartucheira](cartwall.md).

## Atalhos de teclado {#keyboard-shortcuts}

![Definições, Atalhos de teclado: cada ação do leitor com a sua tecla, e Desassociar ao lado das associadas](../../images/guide/settings-shortcuts.png)

Ver [Teclado](keyboard.md).

## MIDI {#midi}

![Definições, MIDI, com o controlo MIDI desativado](../../images/guide/settings-midi.png)

Ative as superfícies de controlo MIDI, veja as portas de entrada e aprenda
um controlo para cada ação do leitor. Ver
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

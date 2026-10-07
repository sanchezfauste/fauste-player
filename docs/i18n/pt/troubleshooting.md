# Resolução de problemas

## Sem som {#no-sound}

1. Abra **Definições → Saídas de áudio** e prima **Testar Main** para o
   leitor. Se ouvir o tom, verifique o fader de volume do leitor.
2. Se não ouvir nada, escolha outro dispositivo ou par de canais. As
   alterações às saídas têm efeito após um reinício: prima **Reiniciar
   agora** nas Definições.
3. No Linux, prefira **PipeWire** ou **PulseAudio** em Definições → Saídas de
   áudio → Sistema de áudio. Partilham a placa de som com outros programas. O
   **ALSA** fala diretamente com a placa e pode encontrá-la ocupada.
4. O **JACK** aparece como indisponível («no output device») quando nenhum
   servidor JACK está em execução. Inicie o servidor (por exemplo, com o
   QjackCtl) e reinicie a aplicação. Defina o servidor JACK para a frequência
   de amostragem das Definições (48 kHz por predefinição): o JACK corre a uma
   só frequência para todos os programas.
5. O **PipeWire** não é oferecido pelos arquivos descarregáveis; estes
   chegam ao PipeWire através do seu serviço PulseAudio, que funciona da
   mesma forma. Está disponível nas versões compiladas com a funcionalidade
   `pipewire`.

## Bit-perfect {#bit-perfect}

- **O indicador BP fica apagado.** Verifique cada condição em
  [Saída bit-perfect](bit-perfect.md#the-bp-badge): volume a 100 %, nenhum
  fade, nada mais nas mesmas saídas, um ficheiro sem perdas que tenha sido
  analisado e um dispositivo a correr à frequência do ficheiro.
- **Um breve silêncio antes de uma faixa.** O dispositivo bit-perfect foi
  aberto de novo à frequência de amostragem da faixa. Mantenha a biblioteca
  a uma só frequência para o evitar.
- **Uma faixa toca reamostrada, e o registo diz que o dispositivo está
  ocupado (Linux).** Para mudar de frequência, a aplicação fecha o
  dispositivo e abre-o de novo. Nesse momento, o servidor de som (PipeWire)
  pode apoderar-se da placa. A aplicação tenta de novo algumas vezes; se a
  placa continuar ocupada, a faixa toca à frequência atual do dispositivo, e
  a faixa seguinte pede de novo a sua frequência. Para dar a placa só à
  aplicação, abra as definições de som do sistema e ponha o perfil dessa
  placa em **Desligado** (ou **Pro Audio**), para que o servidor de som deixe
  em paz o seu dispositivo `hw:`. O número de tentativas e a espera entre
  elas são `tuning.device_busy_retries` e `tuning.device_busy_retry_ms` no
  ficheiro de configuração.
- **O dispositivo toca, mas o indicador BP fica apagado (Windows ou
  macOS).** O acesso exclusivo foi recusado, e o dispositivo toca
  partilhado.
  - Windows: outro programa pode ter o dispositivo em exclusivo, ou o
    controlo exclusivo está desativado nas propriedades Avançadas do
    dispositivo.
  - macOS: outro programa pode ter o dispositivo em hog mode, ou o
    dispositivo oferece as suas frequências apenas como um intervalo contínuo
    (a maioria das interfaces lista frequências fixas).
- **Um dispositivo `hw:` não pode ser aberto (Linux).**
  - Um servidor de som pode estar a segurar a placa. Pare-o, ou configure o
    servidor para deixar essa placa em paz, e reinicie a aplicação.
  - Alguns DAC USB só aceitam amostras de 24 bits empacotadas (`S24_3LE`), que
    a biblioteca de áudio não suporta. Use essa placa através de `plughw:`
    (não é bit-perfect).

### DSD {#dsd}

- **Uma faixa DSD toca convertida embora o dispositivo esteja em DoP ou DSD
  nativo.** O registo diz porquê («DSD converted to PCM» e o motivo) nestas
  causas: o volume do leitor não está a 100 %, algo mais a tocar no
  dispositivo, mais de dois canais, ou um dispositivo que recusa a frequência
  (o DoP precisa da frequência DSD dividida por 16, por exemplo 176,4 kHz para
  DSD64) ou não tem formato de 24 ou 32 bits. Uma faixa que ainda não foi
  analisada converte em silêncio, sem linha no registo: analise-a (Definições
  → Análise) e toque-a de novo.
- **Só a primeira faixa de um álbum DSD sai como DSD.** Essa é a definição
  de mistura predefinida: as faixas que o leitor inicia por si só tocam
  convertidas. Escolha **Manter o DSD e silenciar as outras fontes** em
  Definições → Saídas de áudio para as manter em DSD. Ver
  [DSD](bit-perfect.md#dsd).
- **O cabeçalho mostra DSD mas o conversor toca ruído ou não sincroniza.** O
  conversor não reconhece o DoP (ou o formato nativo). Volte a pôr o
  dispositivo em **Converter para PCM**.
- **Um estalido quando uma faixa DSD começa, para ou deixa o DSD.** O
  conversor precisa de mais silêncio DSD: aumente o **Silêncio DSD** (200 ms
  por predefinição) em Definições → Saídas de áudio, Avançado.
- **Outros leitores ou cartuchos estão silenciosos no dispositivo.** Uma
  faixa DSD está a tocar com **Manter o DSD e silenciar as outras fontes**; o
  indicador **Outras silenciadas** aparece. Voltam a soar quando a faixa
  termina.

## Aviso de «Saída perdida» {#output-lost-alert}

A barra de estado mostra **Saída perdida: &lt;dispositivo&gt;** quando um
dispositivo deixa de responder. Os leitores continuam a contar e a misturar
com um relógio interno, para que a automação não bloqueie. O dispositivo é
tentado de novo a cada 2 segundos e é retomado quando regressa. Volte a
ligar o cabo ou volte a ligar a interface.

### «Saída perdida» que nunca desaparece, com uma saída `hw:` direta {#output-lost-that-never-clears-with-a-direct-hw-output}

Uma placa de som usada através de uma saída ALSA `hw:` direta (por exemplo,
uma saída bit-perfect) é retida só pelo Fauste Player: o servidor de som
(PipeWire ou PulseAudio) não a pode usar ao mesmo tempo. Se outra saída passar
pelo dispositivo predefinido do servidor de som e esse dispositivo
predefinido for a mesma placa, essa saída nunca arranca e fica em **Saída
perdida**. O registo diz «output device opened but never started» uma vez.

Use um caminho por placa: encaminhe todas as saídas dessa placa pelo mesmo
dispositivo `hw:` (com canais diferentes, se necessário), ou escolha outra
placa como saída predefinida do servidor de som nas definições de som do
sistema.

## Uma faixa mostra um ícone de aviso ou um ficheiro com uma cruz {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

O ficheiro está em falta (movido, apagado, desmontado: um ficheiro com uma
cruz) ou não pode ser descodificado (um sinal de aviso). Os leitores
saltam-no. Passe o rato sobre a linha para ver qual dos casos é, e o
caminho do ficheiro.

Um ficheiro em falta é procurado de novo a cada 30 segundos
(`tuning.missing_recheck_ms` no ficheiro de configuração): quando a unidade é
montada ou o ficheiro é posto de volta, a faixa passa a poder tocar por si
só. Um ficheiro que não pode ser descodificado é verificado de novo por si
só no mesmo temporizador, por tamanho e data de modificação: não é
descodificado de novo a menos que um deles tenha mudado, por exemplo quando
uma cópia termina. Para o verificar de imediato, use **Voltar a analisar** no
menu da sua linha, ou **Definições → Análise → Voltar a analisar todas as
faixas** para toda a biblioteca.

## Cortes de áudio {#audio-dropouts}

A barra de estado avisa durante 5 segundos após cada corte que a aplicação
deteta: **P1: cortes de áudio (3)** quando a descodificação de um leitor não
acompanhou o disco (a contagem é da faixa que está a tocar) e
**&lt;dispositivo&gt;: cortes do dispositivo de áudio (2)** quando o
dispositivo de saída falhou um prazo (um xrun). O registo também regista
cada um, no máximo uma linha a cada 10 segundos por tipo, com quantos
aconteceram. Nem todos os sistemas de áudio comunicam xruns (o PulseAudio
não; o modo exclusivo do Windows não).


- Aumente o **tamanho do buffer** nas Definições (e prima **Reiniciar
  agora**).
- No Linux, permita o escalonamento em tempo real. A aplicação pede-o ao
  sistema através do rtkit (D-Bus). Pertencer ao grupo `audio` com um limite
  `rtprio` também funciona.
- Evite unidades de rede para a música que toca no ar.

## «A interface teve um erro» {#the-interface-hit-an-error}

Foi detetado um erro de desenho. O áudio não é afetado. Prima **Reiniciar a
interface**. Comunique-o, por favor, com os registos.

## Registos e relatórios de falhas {#logs-and-crash-reports}

Consulte [Dados e cópias de segurança](data-and-backups.md) para a pasta de
registos. Há um ficheiro de registo por dia, e os últimos 14 são mantidos.
Os relatórios de falhas são guardados como `crash-<hora>.txt`. Defina
`RUST_LOG=debug` no ambiente para mais detalhe. Anexe ambos os ficheiros ao
comunicar um erro.

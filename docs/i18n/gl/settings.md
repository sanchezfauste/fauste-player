# Configuración

Abre **Configuración** na barra superior. Péchase con **Pechar** ou con
`Esc`. A maioría dos cambios aplícanse ao instante e gárdanse
automaticamente.

A xanela ten un só tamaño (900 × 640, máis pequena nunha pantalla pequena)
sexa cal for a sección, e a sección desprázase dentro dela. Cada sección
aliña as súas etiquetas nunha columna.

As seccións Reprodutores, Vúmetros, Análise, Sonoridade e Atallos de teclado teñen un
botón **Restaurar os valores predeterminados** na súa cabeceira. Pide
confirmación e despois restablece só esa sección (Reprodutores conserva o
número de reprodutores e o idioma; Atallos non ten outro botón de
restablecemento). Saídas de audio, Listas, Cartucheira, Biblioteca, MIDI e
Remoto non o teñen.

## Axustes pendentes {#settings-pending}

Todos os axustes se aplican mentres a aplicación funciona. O sistema de
audio, a frecuencia de mostraxe, o tamaño do búfer (tamén o propio dun
dispositivo), as saídas Main e Cue (reprodutores e cartucheira), os
dispositivos bit-perfect e os axustes DSD cambian como se abre un
dispositivo de saída, así que se aplican en canto é seguro:

- Os axustes dun dispositivo aplícanse cando nada soa nese dispositivo:
  ningún reprodutor dirixido a el está en reprodución, facendo un fundido nin
  preescoitando (CUE, tamén cando está retido), e ningún cartucho soa nel.
  Un reprodutor en pausa, ou cunha pista só cargada, non retén un
  dispositivo: a súa pista queda en pausa onde estaba.
- A saída dun reprodutor ou da cartucheira móvese cando ese reprodutor (ou a
  cartucheira) non está reproducindo; os outros reprodutores nos mesmos
  dispositivos non se interrompen.
- Un sistema de audio novo aplícase cando nada soa en ningún sitio.

Un cambio en espera execútase no primeiro momento en que nada do que afecta
soa. Pode ser o oco entre que remata unha pista e comeza a seguinte: entón a
pista seguinte comeza máis tarde polo tempo que tarda o dispositivo en
abrirse de novo. **Aplicar agora** (abaixo) forza o cambio ao instante.

Unha frecuencia ou un búfer dado a un dispositivo só conta cando cambia co
que o dispositivo abre: darlle a un dispositivo o mesmo valor que o global, ou
borrar ese valor, non cambia nada.

Mentres tanto a barra superior mostra **Axustes pendentes**. Pon o punteiro
enriba para ver o que agarda e por que; fai clic para ver a lista, que tamén
mostra calquera cambio que un dispositivo ou unha saída rexeitou (o
dispositivo conserva os seus axustes actuais; o cambio inténtase de novo cando
o cambias ou premes **Aplicar agora**). O pé de Configuración di «Algúns
cambios agardan a que as saídas que afectan queden libres.» e ofrece
**Aplicar agora**; un aviso curto (por exemplo, que se gardou un axuste) pode
ocupar un momento o lugar dese texto.

**Aplicar agora** aplica ao instante todos os cambios de saída en espera.
Cando iso interrompería algo, primeiro lista os dispositivos e o que soa
neles; **Interromper e aplicar** interrompe entón brevemente o audio nesos
dispositivos: o que soaba continúa desde onde estaba despois dun oco curto, e
nada que estivese en pausa ou detido comeza. **Cancelar** (ou `Esc`) segue
agardando.

Os límites e os axustes do motor edítanse no ficheiro de configuración coa
aplicación pechada (consulta [Datos e copias de seguranza](data-and-backups.md)).

## Saídas de audio {#audio-outputs}

Os cambios desta sección aplícanse mentres a aplicación funciona, en canto
nada do que afectan está a soar: consulta [Axustes pendentes](#settings-pending).

![Configuración, Saídas de audio, vista Básico: o selector, o sistema de audio, a frecuencia de mostraxe, o tamaño do búfer e as saídas Main e Cue de cada reprodutor (aquí o sistema silencioso)](../../images/guide/settings-outputs.png)

Arriba, **Mostrar** escolle **Básico** ou **Avanzado**. Básico mostra o
sistema de audio, a frecuencia de mostraxe, o tamaño do búfer e as saídas.
Avanzado engade, para cada dispositivo que usa unha saída, a súa propia
frecuencia e búfer, o interruptor bit-perfect e o modo DSD, e despois os
axustes DSD. Cambiar de vista só mostra ou oculta filas: non se cambia nin
se restablece nada. Cando Básico oculta un axuste que está en uso, unha liña
dío. Cando ningunha saída usa un dispositivo, a súa propia frecuencia e
búfer, o seu interruptor bit-perfect e o seu modo DSD esquécense no seguinte
inicio da aplicación: se unha saída volve usalo despois, parte dos valores
globais. Ata entón, escollelo de novo (por exemplo tras intercambiar dous
dispositivos) conserva eses axustes.

| Axuste | Significado |
|---|---|
| Sistema de audio | A última opción, **Sen saída (silencio)**, non reproduce nada: as liñas de tempo avanzan a ritmo de tempo real sen tarxeta de son (para un computador que non a ten, ou para ensaiar). Linux: PipeWire (nas compilacións que o inclúen), PulseAudio, JACK ou ALSA. Windows: WASAPI, ASIO (nas compilacións que o inclúen) ou JACK. macOS: Core Audio ou JACK. Os sistemas que faltan neste computador, ou sen dispositivo de saída (un servidor JACK que non está en execución), móstranse como non dispoñibles. «Predeterminado do sistema» usa o primeiro dispoñible nesa orde. |
| Frecuencia de mostraxe | A frecuencia á que funciona cada saída salvo que un dispositivo teña a súa propia (Avanzado); os ficheiros convértense a ela con remostraxe de alta calidade. Os dispositivos bit-perfect empezan á súa frecuencia e despois seguen os ficheiros. |
| Tamaño do búfer | Frames por bloque de audio, salvo que un dispositivo teña o seu propio; a latencia resultante móstrase debaixo. As saídas a través dun servidor de son ou dun mesturador do sistema (PulseAudio, o dispositivo ALSA `default`) piden polo menos 50 ms sexa cal for este tamaño, ou todo o que acepte o dispositivo, para que un computador ocupado non se note; a latencia dío entón (`shared_buffer_ms` no ficheiro de configuración) |
| Saídas por reprodutor | Para cada reprodutor, un dispositivo **Main** (en antena) e un **Cue** (preescoita), cada un cun par de canles. Unha tarxeta de son que ofrece varios perfís de saída (ALSA lista frontal, surround, hardware directo…) mostra cada un como *tarxeta — perfil*; dúas entradas que aínda se lerían igual reciben o seu identificador de dispositivo entre corchetes. As interfaces multicanle poden levar varios reprodutores en pares distintos. Unha saída Cue que comparte unha saída de antena (a Main de calquera reprodutor ou a da cartucheira) está permitida: un aviso debaixo dela di que a súa preescoita óese en antena e que pregunta antes mentres algo soa alí. |
| Probar Main / Probar Cue | Reproduce un ton curto (1 kHz en Main, 440 Hz en Cue, 1,5 s, −18 dBFS) na saída escollida, para comprobar a conexión antes de saír en antena |
| Biblioteca | A saída Cue da biblioteca, co seu propio **Probar Cue**: onde se oe a [preescoita](library.md#pre-listening) da xanela da biblioteca (nun teléfono, a da lapela Biblioteca). Sen ela, o Play da barra de preescoita aparece atenuado. En Android a súa opción baleira le *Predeterminado do sistema* en lugar de *Ningunha*: a preescoita e **Probar Cue** soan entón na saída predeterminada (consulta [Android → Biblioteca](android.md#library)). Unha saída da biblioteca nunha saída de antena funciona como o Cue dun reprodutor: un aviso debaixo dela di que a súa preescoita óese en antena e pregunta antes mentres algo soa alí. Está nas vistas Básico e Avanzado, tamén nun teléfono |
| Cartucheira | As saídas Main e Cue da cartucheira. Main toma como predeterminada a saída do sistema. Sen Cue non hai preescoita de cartuchos; un Cue nunha saída de antena funciona como nos reprodutores. |
| Frecuencia de mostraxe: *dispositivo* (Avanzado) | **Global (...)** usa a frecuencia de mostraxe de arriba; un valor dálle a este dispositivo a súa propia frecuencia. Só se ofrecen as frecuencias que indica o dispositivo; unha frecuencia gardada que xa non indica segue na lista cunha nota de que pode non abrirse (entón o dispositivo recorre á frecuencia global). Os valores propios só se aplican aos dispositivos que nomea unha saída, non á saída predeterminada do sistema salvo que unha o faga. |
| Tamaño do búfer: *dispositivo* (Avanzado) | **Global (...)** usa o tamaño do búfer de arriba (co mínimo que pide unha saída compartida, como arriba); un valor dálle a este dispositivo o seu propio, usado tal cal, coa súa latencia debaixo. Un dispositivo que non acepta o seu propio tamaño de búfer recorre ao global, e tamén á frecuencia global cando tampouco acepta a súa propia frecuencia. |
| Bit perfect: *dispositivo* (Avanzado) | Desactivado de forma predeterminada. Un modo especial para a reprodución audiófila, non para programas mesturados: o dispositivo ábrese con acceso exclusivo e segue a frecuencia de mostraxe de cada pista. Os reprodutores cuxa saída Main está nel nunca fan fundidos cruzados nin mesturas, un cambio de frecuencia deixa un silencio curto entre dúas pistas, e o seu volume mantense ao 100 %. O interruptor está desactivado onde o dispositivo non pode dar acceso exclusivo. Consulta [O que custa](bit-perfect.md#what-it-costs). |
| DSD: *dispositivo* (Avanzado) | **Converter a PCM** (o predeterminado), **DoP** ou, en Linux, **DSD nativo**. Cada dispositivo o mostra; só se ofrecen os modos que o dispositivo pode aceptar, e unha liña debaixo di por que os outros non. Consulta [DSD](bit-perfect.md#dsd). |
| Cando outra fonte necesita unha saída DSD (Avanzado) | **Continuar a pista DSD en PCM** (o predeterminado), ou **Manter o DSD e silenciar as outras fontes**. Consulta [DSD](bit-perfect.md#dsd). |
| Silencio DSD (Avanzado) | Silencio enviado antes de que comece un fluxo DSD, despois de que remate e ao pasar a PCM, para que o conversor se enganche sen chasquido; 200 ms de forma predeterminada, de 0 a 2000. |

Unha saída Cue nunca recorre á saída que usa Main. Un Cue que nomea un
dispositivo dun sistema de audio que este computador non ten significa «sen
cue». Un Cue nunha saída de antena (o mesmo dispositivo e canles ca unha Main,
ou solapándoos) está permitido: un aviso debaixo dela di que a súa preescoita
óese en antena e que pregunta antes mentres algo soa alí. Un reprodutor sen
saída Cue ten o botón **CUE** atenuado; ao poñer o punteiro enriba dinos que
escollamos aquí unha saída Cue. O mesmo vale para **Preescoitar no CUE** da
cartucheira.

Se un dispositivo desaparece mentres se reproduce, os reprodutores conservan
as súas liñas de tempo, e o dispositivo ábrese de novo cando volve (consulta
[Resolución de problemas](troubleshooting.md)).

## Reprodutores {#players}

![Configuración, Reprodutores: número de reprodutores, modo predeterminado, duración do fundido, mestura automática, cue-in e cue-out, aviso de fin de pista e idioma](../../images/guide/settings-players.png)

| Axuste | Predeterminado | Significado |
|---|---|---|
| Idioma | Sistema | Idioma da interface |
| Número de reprodutores | 4 | Columnas da pantalla principal (non se pode quitar un reprodutor en antena). O número aplícase ao soltar o control deslizante; un número escrito na caixa aplícase ao premer Enter ou saír da caixa, e Escape cancélao |
| Modo predeterminado | CONT | O modo co que arrancan os reprodutores |
| Duración do fundido | 1000 ms | Usado por Play en antena e por Stop con fundido. Nun reprodutor cuxa saída Main é bit-perfect, só por Stop con fundido |
| Mestura automática no punto MIX | Activada | Superpoñer as pistas no modo continuo. Non se aplica a un reprodutor cuxa saída Main é bit-perfect |
| Usar cue-in e cue-out | Activado | Desactivado: os reprodutores reproducen cada pista do principio ao fin do ficheiro; os marcadores consérvanse e os cartuchos seguen usando os seus. As duracións e os totais das listas seguen o mesmo rango |
| Aviso de fin de pista | 10 s | Cando a conta atrás empeza a pestanexar en vermello |

Mentres a saída Main dun reprodutor é bit-perfect, unha nota debaixo de
**Mestura automática no punto MIX** di que eses reprodutores nunca mesturan
nin fan fundidos cruzados. Ambos os axustes seguen sendo editables: aínda se
aplican aos outros reprodutores. Consulta [O que custa](bit-perfect.md#what-it-costs).

## Vúmetros {#meters}

![Configuración, Vúmetros, co medidor de pico dixital escollido](../../images/guide/settings-meters.png)

Os cambios aplícanse ao instante. Configuración só mostra o que usa o tipo de
medidor escollido: os medidores EBU, DIN e VU teñen a escala, a zona vermella
e o comportamento que fixa a súa norma (só se define o nivel de aliñamento), e
o aliñamento dun medidor K-System é o seu propio 0. Un valor que definas
consérvase para cando volvas escoller ese tipo.

| Axuste | Predeterminado | Significado |
|---|---|---|
| Tipo de medidor | Pico dixital | Como sobe e baixa a barra, e a súa escala, segundo unha norma (ver abaixo) |
| Tempo de subida, Velocidade de caída | 5 ms, 11,8 dB/s | Só para **Personalizado**. O tempo de subida é un tempo de integración: unha ráfaga de ton dese tempo le 2 dB menos; 0 mostra todos os picos. |
| True peak | Desactivado | Só pico dixital, personalizado e K-System. Mide entre mostras, co filtro de sobremostraxe 4× que publica ITU-R BS.1770. Mostra picos que superan 0 dBFS tras a conversión, que un medidor de pico de mostra non ve. Como permite a norma, un chasquido illado dunha soa mostra pode ler ata uns 0,3 dB menos que o seu valor de mostra. |
| Chan da escala | −60 dBFS | O extremo inferior da escala dixital (pico dixital e personalizado). Os outros medidores mostran o rango que dá a súa norma. |
| Retención do pico | 2 s | Só pico dixital, personalizado e K-System: canto tempo permanece aceso o nivel máis alto; 0 desactívao. Os medidores de programa e o VU non teñen retención. |
| Nivel de aliñamento | −18 dBFS | Todos menos o K-System. Marcado na escala (EBU R68). É tamén onde están a marca EBU TEST, a marca DIN −9 e 0 VU. |
| Aviso desde | −9 dBFS | Amarelo desde aquí (máximo permitido EBU), para os medidores de pico dixital e personalizado |
| Perigo desde | −3 dBFS | Vermello desde aquí, para os medidores de pico dixital e personalizado. Os outros póñense vermellos onde o fai a súa escala: VU desde 0 VU, PPM EBU e DIN desde o máximo permitido (EBU +9, DIN 0), K-System desde +4. |
| Lectura de sonoridade | Curto prazo | A sonoridade baixo o medidor: desactivada, momentánea (últimos 400 ms) ou a curto prazo (últimos 3 s), EBU R128 |
| Obxectivo de sonoridade | −23 LUFS | A lectura é verde a ±1 LU (EBU R128) |

| Tipo de medidor | Norma | Comportamento |
|---|---|---|
| Pico dixital | IEC 60268-18 | Mostra todos os picos ao instante; cae 20 dB en 1,7 s |
| PPM EBU | IEC 60268-10 tipo IIb | Os picos máis curtos de aproximadamente 10 ms len menos (unha ráfaga de ton de 10 ms le uns 1,6 dB menos, unha de 0,5 ms uns 18 dB menos), dentro das tolerancias de EBU Tech 3205; cae 24 dB en 2,8 s |
| PPM DIN | IEC 60268-10 tipo I | O mesmo cun tempo de integración de 5 ms; cae 20 dB en 1,5 s |
| VU | IEC 60268-17 | O nivel medio, co movemento de agulla dun VU: 99 % en 300 ms, cun lixeiro sobreimpulso; unha sinusoide le o seu nivel de pico |
| K-20, K-14, K-12 | K-System | Dúas seccións: a media (RMS, 600 ms) como barra sólida e o pico (cae 26 dB en 3 s) atenuado por riba. 0 está 20, 14 ou 12 dB por debaixo da escala completa; verde por debaixo de 0, ámbar de 0 a +4, vermello por riba. K-12 axústase á radiodifusión, K-14 e K-20 a programas con máis dinámica. |
| Personalizado | — | O teu tempo de subida e a túa velocidade de caída |

Cada medidor usa a escala da súa norma, coas súas marcas entre as canles:

| Medidor | Escala |
|---|---|
| Pico dixital, personalizado | −60 … 0 dBFS, marcas cada 10 dB ata −40 e cada 5 dB por riba; os 20 dB superiores ocupan a metade da altura |
| PPM EBU | −12 … +12 arredor do nivel de aliñamento (TEST), cada 4 dB; os niveis máis baixos quedan no extremo inferior |
| PPM DIN | −50 … +5, onde 0 está 9 dB por riba do nivel de aliñamento (−9 dBFS de forma predeterminada) |
| VU | −20 … +3 VU, 0 VU no nivel de aliñamento; a barra móvese en proporción á tensión, como a agulla |
| K-System | desde +20, +14 ou +12 (0 dBFS) ata −60; uniforme en dB ata −24 |

## Análise {#analysis}

![Configuración, Análise: os limiares dos marcadores automáticos](../../images/guide/settings-analysis.png)

Os limiares descritos en [Marcadores e mestura](markers-and-mixing.md).
**Volver analizar todas as pistas** executa a análise de novo para toda a
biblioteca, lendo de novo cada ficheiro; os marcadores manuais consérvanse.
Mentres se executa, o botón aparece atenuado e mostra canto leva feito
(**Analizando… 120 de 340**), así que premelo de novo non comeza outra vez; a
dica de **Analizando N pistas** da barra de estado dá o mesmo progreso. Cambiar
aquí un limiar tamén analiza a biblioteca de novo, e reinicia unha execución
que xa estea en marcha para que ningunha pista conserve marcadores dos valores
antigos.

Despois dunha actualización cuxa análise cambiou, as pistas analizadas pola
versión anterior conservan os seus marcadores e formas de onda, que seguen
funcionando. Ao iniciar, Fauste Player di cantas hai e ofrece **Analizar
agora** ou **Máis tarde**; **Analizar as pistas desactualizadas (N)** aquí fai
o mesmo en calquera momento. As pistas dos reprodutores póñense ao día igualmente,
a medida que se mostran, e tamén as pistas dos cartuchos que non teñen formato
rexistrado (un cartucho só se reproduce bit-perfect cando se coñece o seu
formato). As pistas cuxo ficheiro falta non se contan ata que o ficheiro
volva.

A análise tamén mide a sonoridade de cada ficheiro para a
[normalización de sonoridade](loudness.md). Os ficheiros analizados antes de
que se medise a sonoridade mídense en segundo plano, cando a análise non ten
outra cousa que facer, estea a normalización activada ou non; os seus
marcadores non se tocan. **Volver analizar** (no menú da lista) e **Volver
analizar todas as pistas** miden de novo.

## Sonoridade {#loudness}

![Configuración, Sonoridade: normalización activada, co obxectivo, o teito de true peak, a ganancia máxima e o axuste máximo nos seus valores predeterminados](../../images/guide/settings-loudness.png)

Reproduce cada pista e cada cartucho á mesma sonoridade, cunha ganancia fixa
por pista. A [normalización de sonoridade](loudness.md) explica como se
calcula a ganancia, o distintivo da cabeceira do reprodutor e os axustes por
pista.

| Axuste | Predeterminado | Rango | Significado |
|---|---|---|---|
| Normalizar a sonoridade | Activada | | Normalización activada ou desactivada |
| Obxectivo | −16 LUFS | −36 … −6 LUFS, pasos de 0,5 | A sonoridade integrada á que se leva cada pista |
| Teito de true peak | −1 dBTP | −12 … 0 dBTP, pasos de 0,1 | Unha subida nunca leva o true peak da pista por riba deste valor. As baixadas non teñen límite |
| Ganancia máxima | 12 dB | 0 … 24 dB | A maior ganancia que pode recibir unha pista baixa. As baixadas non teñen límite |
| Axuste máximo | 12 dB | 0 … 24 dB | Ata onde pode chegar o axuste manual de cada pista, cara arriba ou cara abaixo |

Os cambios aplícanse ás pistas que se carguen despois: o que soa conserva a
súa ganancia, e a pista seguinte que agarda en cada reprodutor prepárase de
novo coa nova. Unha nota debaixo dos axustes di cantas pistas aínda agardan
a súa medición; soan sen cambios ata que remate. Un reprodutor ou un
cartucho nunha saída bit-perfect nunca se normaliza. **Restaurar os valores
predeterminados** restablece só esta sección (normalización activada). A
normalización está activada nas instalacións novas; unha configuración
existente conserva o seu axuste actual (toda configuración gardada xa garda
o valor).

## Listas {#playlists}

![Configuración, Listas: o cartafol de música, as listas e as columnas da táboa](../../images/guide/settings-playlists.png)

- **Cartafol de música:** onde comezan os diálogos de ficheiros.
- **Nova lista**, **renomear** (edita o nome e preme Enter; Esc cancela) e
  **eliminar** (icona de papeleira).
- **Importar M3U / PLS…** crea unha lista nova a partir dun ficheiro de lista.
  **M3U** en cada fila expórtaa como M3U8. Consulta [Listas](playlists.md).
- **Columnas da táboa:** que columnas mostran as táboas de pistas e en que
  orde, para todos os reprodutores: unha caixa de selección por columna
  (Título e Dur. non se poden desactivar), frechas arriba e abaixo para as
  mostradas, e **Columnas predeterminadas**. Consulta [Listas](playlists.md).

**Idioma:** unha lista despregable: **Sistema** (segue o sistema operativo),
e despois cada idioma no que está dispoñible a interface, cada un co seu
propio nome (primeiro English, despois por orde alfabética: por exemplo
Español). A interface cambia ao instante. Un idioma do sistema sen tradución
propia usa o máis próximo (o francés de Canadá usa francés, o portugués do
Brasil usa portugués), e inglés se non hai ningún. Un idioma no ficheiro de
configuración que a interface non ten aparece como **Sistema** e segue o
sistema operativo.

A interface en todos os idiomas, inglés e español incluídos, xerouse con IA
e pode conter erros. **Acerca de** leva un aviso xeral que os cobre a todos.
Agradecemos as correccións en forma de issues ou pull requests.

## Cartucheira {#cartwall}

![Configuración, Cartucheira: as páxinas, a grella e o editor do cartucho seleccionado](../../images/guide/settings-cartwall.png)

Páxinas, tamaño da grella, o editor de cartuchos, e importación e exportación
de páxinas de cartuchos. Consulta [Cartucheira](cartwall.md).

## Biblioteca {#library}

Crear, renomear, cambiar a cor, cambiar o tipo, mover (arrastrando) e eliminar
as categorías que agrupan as túas pistas, vixiar un cartafol cunha categoría e
quitar as súas pistas que faltan, sen abrir a xanela da biblioteca. Consulta
[Biblioteca](library.md#categories) e
[Cartafoles vixiados](library.md#watched-folders).

## Atallos de teclado {#keyboard-shortcuts}

![Configuración, Atallos de teclado: cada acción do reprodutor coa súa tecla, e Quitar ao carón das asignadas](../../images/guide/settings-shortcuts.png)

Consulta [Teclado](keyboard.md).

## MIDI {#midi}

![Configuración, MIDI, co control MIDI desactivado](../../images/guide/settings-midi.png)

Activa as superficies de control MIDI, mira os portos de entrada (**Reconectar
portos** ábreos de novo, como faría volver enchufar os controladores) e
aprende un control para cada acción do reprodutor. Consulta
[Superficies de control MIDI](midi.md).

## Remoto {#remote}

![Configuración, Remoto, coa API HTTP a escoitar neste computador](../../images/guide/settings-remote.png)

Control remoto pola rede, para páxinas web, aplicacións de teléfono,
automatización e superficies de control. Consulta
[Control remoto](remote-control.md).

- **Permitir o control remoto por HTTP**, o seu **enderezo** e **porto**, e
  unha liña que di se está a escoitar.
- **Testemuño**, necesario fóra deste computador. **Xerar** crea un aleatorio,
  **Mostrar** revélao e **Copiar** ponio no portapapeis. Aparece un aviso
  cando o enderezo chega máis aló deste computador e non hai testemuño.
- **Páxinas web que poden usar a API**: unha orixe por liña.
- **Permitir o control por OSC**, o seu **enderezo** e **porto**, e os
  **emisores permitidos** (enderezos ou subredes, un por liña).
- **Publicar os tempos cada**: con que frecuencia se envían os tempos
  transcorrido e restante mentres algo soa.

Os campos de texto e os números aplícanse ao saír deles, o que inclúe abrir
outra sección ou pechar Configuración; Esc cancela o que estabas escribindo.
Un valor non válido corríxese, e o campo mostra o que se conservou.

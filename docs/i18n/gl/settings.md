# Configuración

Abre **Configuración** na barra superior. Péchase con **Pechar** ou con
`Esc`. A maioría dos cambios aplícanse ao instante e gárdanse
automaticamente.

A xanela ten un só tamaño (900 × 640, máis pequena nunha pantalla pequena)
sexa cal for a sección, e a sección desprázase dentro dela. Cada sección
aliña as súas etiquetas nunha columna.

As seccións Reprodutores, Vúmetros, Análise e Atallos de teclado teñen un
botón **Restaurar os valores predeterminados** na súa cabeceira. Pide
confirmación e despois restablece só esa sección (Reprodutores conserva o
número de reprodutores e o idioma; Atallos non ten outro botón de
restablecemento). Saídas de audio, Listas, Cartucheira, MIDI e Remoto non o
teñen.

## Reinicio pendente {#restart-pending}

Algúns cambios só teñen efecto cando a aplicación se inicia de novo: o
sistema de audio, a frecuencia de mostraxe, o tamaño do búfer (tamén o propio
dun dispositivo), as saídas Main e Cue (reprodutores e cartucheira), os
dispositivos bit-perfect e os axustes DSD. O número de reprodutores non é un
deles: aplícase ao instante.

Unha frecuencia ou un búfer dado a un dispositivo só conta cando cambia co
que o dispositivo abre: darlle a un dispositivo o mesmo valor que o global, ou
borrar ese valor, non queda pendente.

Os límites e os axustes do motor tamén se aplican no seguinte inicio, pero
edítanse no ficheiro de configuración coa aplicación pechada (consulta
[Datos e copias de seguranza](data-and-backups.md)), así que nunca aparecen
como pendentes.

Mentres hai algo destes á espera, o pé de Configuración di «Algúns cambios
aplícanse despois de reiniciar.» e ofrece **Reiniciar agora**, e a barra
superior mostra unha pílula **Reinicio pendente**. Pon o punteiro sobre a
pílula para ver o que está á espera. Un aviso curto (por exemplo, que se
gardou un axuste) pode ocupar un momento o lugar do texto do pé; **Reiniciar
agora** permanece. Ambos fan o mesmo:

- Cando non hai nada en antena, **Reiniciar agora** (ou a pílula) reinicia ao
  instante.
- Cando hai algo en antena, aparece a xanela que lista o que está a soar, con
  **Deter e reiniciar** ou **Cancelar**.

A sesión gárdase primeiro e o audio e o control MIDI paran, despois a
aplicación inicia de novo co mesmo cartafol de datos (`FAUSTE_HOME`), e nada
sae en antena por si só despois. Se a aplicación non pode iniciarse de novo
(nun Flatpak, tamén cando a nova non arranca a tempo), dío; iníciaa desde o
menú de aplicacións.

## Saídas de audio {#audio-outputs}

Os cambios desta sección agardan a un reinicio: consulta
[Reinicio pendente](#restart-pending).

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
| Tamaño do búfer | Frames por bloque de audio, salvo que un dispositivo teña o seu propio; a latencia resultante móstrase debaixo |
| Saídas por reprodutor | Para cada reprodutor, un dispositivo **Main** (en antena) e un **Cue** (preescoita), cada un cun par de canles. Unha tarxeta de son que ofrece varios perfís de saída (ALSA lista frontal, surround, hardware directo…) mostra cada un como *tarxeta — perfil*; dúas entradas que aínda se lerían igual reciben o seu identificador de dispositivo entre corchetes. As interfaces multicanle poden levar varios reprodutores en pares distintos. |
| Probar Main / Probar Cue | Reproduce un ton curto (1 kHz en Main, 440 Hz en Cue, 1,5 s, −18 dBFS) na saída escollida, para comprobar a conexión antes de saír en antena |
| Cartucheira | As saídas Main e Cue da cartucheira. Main toma como predeterminada a saída do sistema. Sen Cue non hai preescoita de cartuchos. |
| Frecuencia de mostraxe: *dispositivo* (Avanzado) | **Global (...)** usa a frecuencia de mostraxe de arriba; un valor dálle a este dispositivo a súa propia frecuencia. Só se ofrecen as frecuencias que indica o dispositivo; unha frecuencia gardada que xa non indica segue na lista cunha nota de que pode non abrirse (entón o dispositivo recorre á frecuencia global). Os valores propios só se aplican aos dispositivos que nomea unha saída, non á saída predeterminada do sistema salvo que unha o faga. |
| Tamaño do búfer: *dispositivo* (Avanzado) | **Global (...)** usa o tamaño do búfer de arriba; un valor dálle a este dispositivo o seu propio, coa súa latencia debaixo. Un dispositivo que non acepta o seu propio tamaño de búfer recorre ao global, e tamén á frecuencia global cando tampouco acepta a súa propia frecuencia. |
| Bit perfect: *dispositivo* (Avanzado) | Un dispositivo bit-perfect ábrese con acceso exclusivo e segue a frecuencia de mostraxe de cada ficheiro mentres non soa nada nel. O interruptor está desactivado onde o dispositivo non pode dar acceso exclusivo. Consulta [Saída bit-perfect](bit-perfect.md). |
| DSD: *dispositivo* (Avanzado) | **Converter a PCM** (o predeterminado), **DoP** ou, en Linux, **DSD nativo**. Cada dispositivo o mostra; só se ofrecen os modos que o dispositivo pode aceptar, e unha liña debaixo di por que os outros non. Consulta [DSD](bit-perfect.md#dsd). |
| Cando outra fonte necesita unha saída DSD (Avanzado) | **Continuar a pista DSD en PCM** (o predeterminado), ou **Manter o DSD e silenciar as outras fontes**. Consulta [DSD](bit-perfect.md#dsd). |
| Silencio DSD (Avanzado) | Silencio enviado antes de que comece un fluxo DSD, despois de que remate e ao pasar a PCM, para que o conversor se enganche sen chasquido; 200 ms de forma predeterminada, de 0 a 2000. |

Unha saída Cue nunca recorre á saída que usa Main, para que a preescoita nunca
saia en antena. Un Cue que nomea un dispositivo dun sistema de audio que este
computador non ten, ou a mesma saída (dispositivo e canles) que Main,
significa «sen cue». Cando unha saída Cue é a mesma que a súa saída Main, un
aviso debaixo dela dío. Un reprodutor sen saída Cue, ou co seu Cue na súa
saída Main, ten o botón **CUE** atenuado; ao poñer o punteiro enriba dinos
que escollamos aquí unha saída Cue. O mesmo vale para **Preescoitar no CUE**
da cartucheira.

Se un dispositivo desaparece mentres se reproduce, os reprodutores conservan
as súas liñas de tempo, e o dispositivo ábrese de novo cando volve (consulta
[Resolución de problemas](troubleshooting.md)).

## Reprodutores {#players}

![Configuración, Reprodutores: número de reprodutores, modo predeterminado, duración do fundido, mestura automática, cue-in e cue-out, aviso de fin de pista e idioma](../../images/guide/settings-players.png)

| Axuste | Predeterminado | Significado |
|---|---|---|
| Idioma | Sistema | Idioma da interface |
| Número de reprodutores | 4 | Columnas da pantalla principal (non se pode quitar un reprodutor en antena) |
| Modo predeterminado | CONT | O modo co que arrancan os reprodutores |
| Duración do fundido | 1000 ms | Usado por Play en antena e por Stop con fundido |
| Mestura automática no punto MIX | Activada | Superpoñer as pistas no modo continuo |
| Usar cue-in e cue-out | Activado | Desactivado: os reprodutores reproducen cada pista do principio ao fin do ficheiro; os marcadores consérvanse e os cartuchos seguen usando os seus. As duracións e os totais das listas seguen o mesmo rango |
| Aviso de fin de pista | 10 s | Cando a conta atrás empeza a pestanexar en vermello |

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
biblioteca; os marcadores manuais consérvanse.

Despois dunha actualización cuxa análise cambiou, as pistas analizadas pola
versión anterior conservan os seus marcadores e formas de onda, que seguen
funcionando. Ao iniciar, Fauste Player di cantas hai e ofrece **Analizar
agora** ou **Máis tarde**; **Analizar as pistas desactualizadas (N)** aquí fai
o mesmo en calquera momento. As pistas dos reprodutores póñense ao día igualmente,
a medida que se mostran, e tamén as pistas dos cartuchos que non teñen formato
rexistrado (un cartucho só se reproduce bit-perfect cando se coñece o seu
formato). As pistas cuxo ficheiro falta non se contan ata que o ficheiro
volva.

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

O inglés e o español están escritos a man. As outras traducións xeráronse con
IA e poden conter erros; cando se usa unha delas, **Acerca de** dío. Agradecemos
as correccións de falantes nativos en forma de issues ou pull requests.

## Cartucheira {#cartwall}

![Configuración, Cartucheira: as páxinas, a grella e o editor do cartucho seleccionado](../../images/guide/settings-cartwall.png)

Páxinas, tamaño da grella, o editor de cartuchos, e importación e exportación
de páxinas de cartuchos. Consulta [Cartucheira](cartwall.md).

## Atallos de teclado {#keyboard-shortcuts}

![Configuración, Atallos de teclado: cada acción do reprodutor coa súa tecla, e Quitar ao carón das asignadas](../../images/guide/settings-shortcuts.png)

Consulta [Teclado](keyboard.md).

## MIDI {#midi}

![Configuración, MIDI, co control MIDI desactivado](../../images/guide/settings-midi.png)

Activa as superficies de control MIDI, mira os portos de entrada e aprende un
control para cada acción do reprodutor. Consulta
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

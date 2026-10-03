#!/usr/bin/env bash
# Generates stand-in audio for screenshots: 16 songs longer than 3 minutes,
# named like music ("Artist – Title.flac"), and 6 short carts (WAV). Each
# song has its own envelope (modulation rate, depth and shape, fades and
# quiet passages), so that no two waveforms look alike. They are pink noise
# and a sine, amplitude-modulated, through a limiter. The songs carry title,
# artist, album, date and genre tags and a front cover (a generated
# gradient); the carts carry title and artist as a RIFF LIST/INFO chunk
# (INAM, IART). The analysis reads both.
#
#   scripts/site/tones.sh <music dir> <carts dir>
#
# Existing files are kept, so a second run is quick. demo_session gives the
# carts their type by length (jingle, effect, spot, in turn), so the names
# below follow that order. Needs ffmpeg, and ImageMagick for the covers.
set -euo pipefail

music=${1:?usage: tones.sh <music dir> <carts dir>}
carts=${2:?usage: tones.sh <music dir> <carts dir>}
mkdir -p "$music" "$carts"
for tool in ffmpeg convert; do
    command -v "$tool" >/dev/null || { echo "tones.sh: $tool is needed" >&2; exit 1; }
done

# tone <file> <secs> <sine Hz> <sine weight> <envelope> <fade in> <fade out> [cover]
# The envelope is an ffmpeg expression of t (seconds), evaluated per frame.
# The tags come from the array `tags` (ffmpeg -metadata arguments). The
# extension chooses the format: .flac (with the cover) or .wav.
tone() {
    local file=$1 secs=$2 freq=$3 weight=$4 env=$5 fin=$6 fout=$7 cover=${8:-}
    [[ -s $file ]] && return 0
    local seed=$(( ${#file} * 7919 + secs ))
    local picture=() codec=(-c:a pcm_s16le -f wav)
    if [[ $file == *.flac ]]; then
        codec=(-c:a flac -sample_fmt s16 -f flac)
        [[ -n $cover ]] && picture=(-i "$cover" -map 3:v -c:v png -disposition:v attached_pic
            -metadata:s:v comment="Cover (front)")
    fi
    ffmpeg -nostdin -loglevel error -y \
        -f lavfi -i "anoisesrc=d=${secs}:c=pink:r=44100:a=0.6:s=${seed}" \
        -f lavfi -i "anoisesrc=d=${secs}:c=pink:r=44100:a=0.6:s=$((seed + 1))" \
        -f lavfi -i "sine=f=${freq}:r=44100:d=${secs}" \
        "${picture[@]:0:2}" \
        -filter_complex "[0][1]join=inputs=2:channel_layout=stereo[n];[2]pan=stereo|c0=c0|c1=c0[s];[n][s]amix=inputs=2:weights='1 ${weight}':normalize=0,volume='${env}':eval=frame,afade=t=in:d=${fin},afade=t=out:st=$(( secs - fout )):d=${fout},volume=9dB,alimiter=limit=0.89:level=false[a]" \
        -map '[a]' "${picture[@]:2}" "${tags[@]}" "${codec[@]}" "$file.part"
    mv "$file.part" "$file"
}

# cover <file> <colour> <colour> <n>: a two-colour gradient, swirled and
# turned by an amount of its own, 300 px.
cover() {
    [[ -s $1 ]] && return 0
    convert -size 424x424 "gradient:$2-$3" -swirl $(( 90 + $4 * 37 % 270 )) \
        -rotate $(( $4 * 53 % 360 )) +repage -gravity center -crop 300x300+0+0 +repage \
        -depth 8 -strip "$1"
}

# Envelope shapes: s = sine swell, q = square (verse/chorus blocks),
# w = saw (slow builds), p = pulses; P = period in seconds, D = depth (0-1),
# and an optional quiet passage from A to B seconds.
envelope() {
    local shape=$1 period=$2 depth=$3 a=$4 b=$5 base
    case $shape in
        s) base="(1-${depth}*0.5*(1+sin(2*PI*t/${period})))" ;;
        q) base="(1-${depth}*lt(mod(t,${period}),${period}*0.45))" ;;
        w) base="(1-${depth}+${depth}*mod(t,${period})/${period})" ;;
        p) base="(1-${depth}*pow(abs(sin(PI*t/${period})),6))" ;;
    esac
    echo "${base}*if(between(t,${a},${b}),0.12,1)"
}

covers=$music/.covers
mkdir -p "$covers"
i=0
while IFS='|' read -r artist title secs freq weight shape period depth a b fin fout album year genre c1 c2; do
    [[ -z $artist ]] && continue
    cover "$covers/$i.png" "$c1" "$c2" "$((i + 11))"
    tags=(-metadata title="$title" -metadata artist="$artist" -metadata album="$album"
        -metadata date="$year" -metadata genre="$genre")
    tone "$music/$artist – $title.flac" "$secs" "$freq" "$weight" \
        "$(envelope "$shape" "$period" "$depth" "$a" "$b")" "$fin" "$fout" "$covers/$i.png"
    i=$((i + 1))
done <<'EOF'
Harbor Lights|Northbound|212|220|0.30|s|24|0.70|96|104|0.5|6|Open Water|2021|Indie|#1d3557|#a8dadc
Mara Quinn|Paper Moons|197|330|0.20|q|32|0.55|150|158|4|3|Paper Moons|2019|Pop|#6d597a|#e56b6f
The Lantern Club|Slow Parade|238|196|0.40|w|60|0.85|0|0|10|12|Night Garden|2017|Jazz|#264653|#e9c46a
Ivo Brandt|Coastline|205|262|0.25|p|9|0.80|60|72|1|2|Salt Roads|2022|Folk|#003049|#fcbf49
Selene Okafor|Glass Rivers|224|440|0.15|s|7|0.45|120|126|8|8|Glass Rivers|2020|Soul|#3d405b|#f2cc8f
Copper Fields|Midnight Ferry|191|147|0.35|q|18|0.75|40|50|0.2|1|Ferry Lights|2016|Rock|#5f0f40|#fb8b24
Nadia Voss|Open Windows|249|294|0.20|w|25|0.60|180|192|3|15|Rooms|2023|Electronic|#22223b|#9a8c98
Low Tide Society|Salt and Static|201|110|0.45|s|50|0.90|0|0|15|5|Static Bloom|2018|Ambient|#0b132b|#5bc0be
Elias Moreno|September Light|216|523|0.10|p|20|0.65|100|110|2|10|Late Summer|2015|Latin|#7f5539|#ede0d4
Pale Orchard|Wildfire Road|187|175|0.30|q|11|0.40|70|74|0.3|4|Wildfire Road|2021|Country|#283618|#dda15e
June Halloway|Small Hours|233|349|0.25|s|15|0.80|30|45|6|20|Small Hours|2024|Singer-songwriter|#2b2d42|#ef233c
Radio Kites|Weather Report|208|247|0.35|w|13|0.70|140|146|1|3|Forecast|2019|Indie|#14213d|#fca311
Tomas Reyes|Blue Junction|195|392|0.20|p|40|0.85|0|0|5|7|Blue Junction|2014|Blues|#03045e|#00b4d8
Velvet Compass|Northern Line|243|165|0.40|q|45|0.65|200|210|2|9|True North|2020|Synth-pop|#3c096c|#ff9e00
Ada Lindqvist|Afterglow|219|587|0.10|s|90|0.75|110|118|12|14|Afterglow|2022|Pop|#590d22|#ffccd5
Saltmarsh|Long Way Home|228|131|0.45|w|38|0.50|55|60|0.5|25|Tidelines|2013|Folk|#344e41|#a3b18a
EOF

while IFS='|' read -r artist title secs freq weight shape period depth fin fout; do
    [[ -z $artist ]] && continue
    tags=(-metadata title="$title" -metadata artist="$artist")
    tone "$carts/$artist – $title.wav" "$secs" "$freq" "$weight" \
        "$(envelope "$shape" "$period" "$depth" 0 0)" "$fin" "$fout"
done <<'EOF'
Jingle|Station ID|5|880|0.6|s|2|0.5|0.1|1
Effect|Whoosh|7|0|0.0|w|7|0.9|0.1|2
Spot|Riverside Bakery|20|440|0.4|p|5|0.6|0.5|2
Jingle|Morning Show|25|740|0.6|w|3|0.7|0.2|2
Effect|Applause|30|0|0.0|s|9|0.4|1|3
Spot|City Theatre|45|330|0.4|q|6|0.5|1|2
EOF

echo "tones.sh: $i songs in $music, carts in $carts"

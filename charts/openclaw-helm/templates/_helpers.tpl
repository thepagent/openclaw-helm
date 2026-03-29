{{/*
Expand the name of the chart.
*/}}
{{- define "openclaw-helm.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "openclaw-helm.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "openclaw-helm.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "openclaw-helm.labels" -}}
helm.sh/chart: {{ include "openclaw-helm.chart" . }}
{{ include "openclaw-helm.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "openclaw-helm.selectorLabels" -}}
app.kubernetes.io/name: {{ include "openclaw-helm.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Container image reference. Uses digest when set, otherwise tag.
Digest without algorithm prefix (e.g. bare hex) gets sha256: auto-prepended.
*/}}
{{- define "openclaw-helm.image" -}}
{{- if .Values.image.digest -}}
{{- $d := .Values.image.digest | trim -}}
{{- if not (contains ":" $d) -}}
{{- $d = printf "sha256:%s" $d -}}
{{- end -}}
{{ .Values.image.repository }}@{{ $d }}
{{- else -}}
{{ .Values.image.repository }}:{{ .Values.image.tag }}
{{- end -}}
{{- end }}

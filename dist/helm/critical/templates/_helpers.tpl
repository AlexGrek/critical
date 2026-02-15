{{/*
Expand the name of the chart.
*/}}
{{- define "critical.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "critical.fullname" -}}
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
{{- define "critical.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "critical.labels" -}}
helm.sh/chart: {{ include "critical.chart" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
API selector labels
*/}}
{{- define "critical.api.selectorLabels" -}}
app.kubernetes.io/name: {{ include "critical.name" . }}-api
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: api
{{- end }}

{{/*
Frontend selector labels
*/}}
{{- define "critical.frontend.selectorLabels" -}}
app.kubernetes.io/name: {{ include "critical.name" . }}-frontend
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: frontend
{{- end }}

{{/*
ArangoDB selector labels
*/}}
{{- define "critical.arangodb.selectorLabels" -}}
app.kubernetes.io/name: {{ include "critical.name" . }}-arangodb
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: database
{{- end }}

{{/*
Get the secret name
*/}}
{{- define "critical.secretName" -}}
{{- if .Values.existingSecret }}
{{- .Values.existingSecret }}
{{- else }}
{{- include "critical.fullname" . }}
{{- end }}
{{- end }}

{{/*
Get the DB connection string (auto-generate if internal ArangoDB is used)
*/}}
{{- define "critical.dbConnectionString" -}}
{{- if .Values.config.dbConnectionString }}
{{- .Values.config.dbConnectionString }}
{{- else }}
{{- printf "http://%s-arangodb:%d" (include "critical.fullname" .) (int .Values.arangodb.port) }}
{{- end }}
{{- end }}

{{/*
Internal API URL (for pod-to-pod communication, e.g. SSR loaders)
*/}}
{{- define "critical.internalApiUrl" -}}
{{- printf "http://%s-api:80" (include "critical.fullname" .) }}
{{- end }}

{{/*
API image
*/}}
{{- define "critical.api.image" -}}
{{- printf "%s:%s" .Values.api.image.repository (default .Chart.AppVersion .Values.api.image.tag) }}
{{- end }}

{{/*
Frontend image
*/}}
{{- define "critical.frontend.image" -}}
{{- printf "%s:%s" .Values.frontend.image.repository (default .Chart.AppVersion .Values.frontend.image.tag) }}
{{- end }}

{{/*
Service account name
*/}}
{{- define "critical.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "critical.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

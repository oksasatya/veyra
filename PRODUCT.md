# Veyra — Product Context

**Register:** product (design serves the task; app UI).

## What it is

Veyra is a self-hosted vehicle-management app. The mobile client (Flutter, iOS + Android) lets an
individual owner track each vehicle's lifecycle: service history, fuel consumption, expenses,
maintenance reminders, and documents, with a per-vehicle dashboard summary. It consumes the Veyra
Rust REST API.

## Who uses it

A single owner-operator managing one to a handful of personal vehicles. Hands-on, detail-oriented,
checks the app in short bursts: after a fill-up, after a service, when a reminder is due. Often
one-handed, outdoors, in variable light (a workshop, a parking lot, a fuel station at night).

## Primary jobs

1. See the state of a vehicle at a glance (odometer, costs to date, what's due soon).
2. Log an event fast: a fuel fill-up, a service, an expense.
3. Track maintenance reminders by date or odometer and mark them done.
4. Keep document references (STNK, BPKB, insurance) and their expiry in one place.

## Platforms

iOS + Android from one Flutter codebase. Honor each platform's chrome: iOS large titles + bottom tab
bar; Android Material 3 top app bar + bottom nav + FAB. Adaptive, not lowest-common-denominator.

## Tone

Confident, technical, calm. The tool disappears into the task. Earned familiarity over novelty:
standard affordances, consistent component vocabulary, color reserved for actions and state.

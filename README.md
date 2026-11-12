# HypeBot

> NOTE: This is an example plugin for [Lorealm](https://github.com/Lorealm/Lorealm).
> It's a good starting point if you're looking for examples.

A little robot that sits on top of your message bar and reacts to the story

## Developing

```sh
lorealm-plugin build
```

Then, in Lorealm, install it with "Add from source" and this folder.

## Releasing

```sh
lorealm-plugin stage 0.2.0   # sets the version, commits it and tags it
lorealm-plugin publish       # pushes the tag, which builds the release
```

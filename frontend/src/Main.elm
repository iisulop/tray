module Main exposing (main)

import Bootstrap.CDN as CDN
import Bootstrap.Grid as Grid
import Browser
import Debug exposing (toString)
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Http
import Json.Decode as Decode exposing (Decoder, string)
import Json.Decode.Pipeline exposing (required)
import Json.Encode as E

main : Program () Model Msg
main =
  Browser.element {
    init = init
  , update = update
  , subscriptions = subscriptions
  , view = view
  }

init : () -> ( Model, Cmd Msg )
init _ =
    (
        Tray
        , Cmd.none
    )

type VoteType
    = VoteSucceeded
    | VoteFailed String

type Model
    = Tray
    | Vote VoteType

type alias VoteTarget = Int

type Msg
    = DisplayImages
    | GiveVote VoteTarget
    | GotVote (Result Http.Error ())

update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GiveVote number ->
            (model, Http.post
                { url = "http://localhost:3030/vote/"
                , body = Http.jsonBody (E.object [("number", E.int number)])
                , expect = Http.expectWhatever GotVote
                }
            )
        GotVote result ->
            case result of
                Ok assignment ->
                    (Vote VoteSucceeded, Cmd.none)
                Err err -> 
                    (Vote (VoteFailed (toString err)), Cmd.none)
        DisplayImages ->
            (Tray, Cmd.none)

view : Model -> Html Msg
view model =
  let 
      info = case model of
        Tray -> "kuvia"
        Vote VoteSucceeded -> "vote succeeded"
        Vote (VoteFailed err) -> "vote failed: " ++ err
  in
  Grid.container []
    [ CDN.stylesheet
    , Grid.row []
        [ Grid.col []
            [ div [] [ text info ]
            , button [ onClick (GiveVote 1) ] [ text "New assignment" ]
            ]
        ]
    ]

  --div []
    --[ div [] [ text info ]
    --, button [ onClick (GiveVote 1) ] [ text "New assignment" ]
    --]

subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none

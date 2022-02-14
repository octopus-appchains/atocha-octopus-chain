
Action: Submission->atochaFinace->preStorage
Event: atochaFinace.PreStorage
Result: Storage->atochaFinace->storageLedger

Create a puzzle
Action: Submission->atochaModule->createPuzzle
Event: atochaModule.PuzzleCreated
Result: Storage->atochaModule->puzzleInfo

Action: Submission->atochaModule->additionalSponsorship
Event: atochaModule.AdditionalSponsorship
Result: Storage->atochaFinace->atoFinanceLedger

Action: Submission->atochaModule->answerPuzzle
Event: atochaModule.AnswerCreated (ANSWER_HASH_IS_MISMATCH || ANSWER_HASH_IS_MATCH)
Result: Storage->atochaModule->puzzleInfo

Action: Submission->atochaModule->commitChallenge
Event: atochaFinace.ChallengeDeposit
Result: Storage->atochaFinace->puzzleChallengeInfo

Action: Submission->atochaModule->challengeCrowdloan
Event: atochaFinace.ChallengeStatusChange
Result: Storage->atochaFinace->puzzleChallengeInfo

Action: Submission->atochaModule->takeAnswerReward
Event: atochaFinace.TakeTokenReward && atochaFinace.TakePointReward
Result: Storage->atochaFinace->puzzleChallengeInfo

Action: 
Storage->atochaFinace->pointExchangeInfo

Action: Submission->atochaFinace->applyPointReward
Event: Event::ApplyPointReward
Result: Storage->atochaFinace->pointExchangeInfo

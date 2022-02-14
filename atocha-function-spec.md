
Action: Submission->atochaFinace->preStorage
Event: atochaFinace.PreStorage
Result: Storage->atochaFinace->storageLedger

### Create a puzzle ###
Action: Submission->atochaModule->createPuzzle
Event: atochaModule.PuzzleCreated
Result: Storage->atochaModule->puzzleInfo

### Sponsor a puzzle ###
Action: Submission->atochaModule->additionalSponsorship
Event: atochaModule.AdditionalSponsorship
Result: Storage->atochaFinace->atoFinanceLedger

### Solve a puzzle ###
Action: Submission->atochaModule->answerPuzzle
Event: atochaModule.AnswerCreated (ANSWER_HASH_IS_MISMATCH || ANSWER_HASH_IS_MATCH)
Result: Storage->atochaModule->puzzleInfo

### Challenge a puzzle - create a challenge ###
Action: Submission->atochaModule->commitChallenge
Event: atochaFinace.ChallengeDeposit
Result: Storage->atochaFinace->puzzleChallengeInfo

### Challenge a puzzle - join a challenge ###
Action: Submission->atochaModule->challengeCrowdloan
Event: atochaFinace.ChallengeStatusChange
Result: Storage->atochaFinace->puzzleChallengeInfo

### Apply for puzzle reward ###
Action: Submission->atochaModule->takeAnswerReward
Event: atochaFinace.TakeTokenReward && atochaFinace.TakePointReward
Result: Storage->atochaFinace->puzzleChallengeInfo

### Apply for puzzle reward ###
Action: 
Storage->atochaFinace->pointExchangeInfo

### Apply for weekly reward ###
Action: Submission->atochaFinace->applyPointReward
Event: Event::ApplyPointReward
Result: Storage->atochaFinace->pointExchangeInfo

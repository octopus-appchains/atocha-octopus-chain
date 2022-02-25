# Atocha Appchain Function Spec
For all players including puzzle creators, puzzle solvers, puzzle sponsors and puzzle challengers. 

### PreCreate a puzzle
- Action: Submission->atochaFinace->preStorage
- Event: atochaFinace.PreStorage
- Result: Storage->atochaFinace->storageLedger

### Create a puzzle
- Action: Submission->atochaModule->createPuzzle
- Event: atochaModule.PuzzleCreated && atochaFinace->PuzzleDeposit
- Result: Storage->atochaModule->puzzleInfo

### Sponsor a puzzle
- Action: Submission->atochaModule->additionalSponsorship
- Event: atochaModule.AdditionalSponsorship
- Result: Storage->atochaFinace->atoFinanceLedger

### Answer a puzzle
- Action: Submission->atochaModule->answerPuzzle
- Event: atochaModule.AnswerCreated (ANSWER_HASH_IS_MISMATCH || ANSWER_HASH_IS_MATCH)
- Result: Storage->atochaModule->puzzleInfo

### Create a puzzle challenge
- Action: Submission->atochaModule->commitChallenge
- Event: atochaFinace.ChallengeDeposit || council.Proposed
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Join a puzzle challenge
- Action: Submission->atochaModule->challengeCrowdloan
- Event: atochaFinace.ChallengeStatusChange && atochaFinace.ChallengeDeposit || council.Proposed
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Claim for puzzle challenge deposit refund
- Action: Submission->atochaModule->challengePullOut
- Event: atochaFinace.ChallengeStatusChange
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Claim for puzzle reward
- Action: Submission->atochaModule->takeAnswerReward
- Event: atochaFinace.TakeTokenReward && atochaFinace.TakePointReward
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Check point ranking
- Result: Storage->atochaFinace->pointExchangeInfo

### Claim for weekly point ranking reward
- Action: Submission->atochaFinace->applyPointReward
- Event: atochaFinace.applyPointReward
- Result: Storage->atochaFinace->pointExchangeInfo


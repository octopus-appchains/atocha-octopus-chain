# Atocha Appchain Function Spec
For all players including puzzle creators, puzzle solvers, puzzle sponsors and puzzle challengers. 

### PreCreate a puzzle
- Action: Submission->atochaFinace->preStorage
- Event: atochaFinace.PreStorage
- Result: Storage->atochaFinace->storageLedger

### Create a puzzle
- Action: Submission->atochaModule->createPuzzle
- Event: atochaModule.PuzzleCreated && atochaFinace.PuzzleDeposit
- Result: Storage->atochaModule->puzzleInfo

### Sponsor a puzzle
- Action: Submission->atochaModule->additionalSponsorship
- Event: atochaFinace.PuzzleDeposit
- Result: Storage->atochaFinace->atoFinanceLedger

### Solve a puzzle
- Action: Submission->atochaModule->answerPuzzle
- Event: atochaModule.AnswerCreated (ANSWER_HASH_IS_MATCH) && atochaModule.PuzzleStatusChange (PUZZLE_STATUS_IS_SOLVED) && atochaModule.AnnouncePuzzleChallengeDeadline
- Result: Storage->atochaModule->puzzleInfo

### Make an initial deposit of a challenge
- Action: Submission->atochaModule->commitChallenge
- Event:<br/>
atochaFinace.ChallengeDeposit && atochaFinace.ChallengeRaisePeriodDeadline<br/>
or<br/>
atochaFinace.ChallengeDeposit && atochaFinace.ChallengeRaisePeriodDeadline && council.Proposed<br/>
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Make further deposit of a challenge
- Action: Submission->atochaModule->challengeCrowdloan
- Event:<br/>
atochaFinace.ChallengeDeposit<br/>
or<br/>
atochaFinace.ChallengeDeposit && atochaFinace.ChallengeStatusChange && council.Proposed<br/>
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Claim for puzzle challenge deposit refund when challenge failed
- Action: Submission->atochaModule->challengePullOut
- Event: atochaFinace.ChallengeStatusChange
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Claim for puzzle reward
- Action: Submission->atochaModule->takeAnswerReward
- Event: atochaFinace.TakeTokenReward && atochaFinace.TakePointReward
- Result: Storage->atochaFinace->puzzleChallengeInfo

### Claim for weekly point ranking reward
- Action: Submission->atochaFinace->applyPointReward
- Event: atochaFinace.applyPointReward
- Result: Storage->atochaFinace->pointExchangeInfo

### Check for a player's current points
- Result: Storage->atochaFinace->atoPointLedger
 
### Check for current point ranking for all players
- Result: Storage->atochaFinace->pointExchangeInfo

### Check for configuration parameters
- Storage->atochaModule->atoConfig()
- Storage->atochaFinace->atoConfig()

# Atocha Appchain Function Spec
For all players including puzzle creators, puzzle solvers, puzzle sponsors and puzzle challengers. 

### PreCreate a puzzle
- Action: Submission->atochaFinance->preStorage
- Event: atochaFinance.PreStorage
- Result: Storage->atochaFinance->storageLedger

### Create a puzzle
- Action: Submission->atochaModule->createPuzzle
- Event: atochaModule.PuzzleCreated && atochaFinance.PuzzleDeposit
- Result: Storage->atochaModule->puzzleInfo

### Sponsor a puzzle
- Action: Submission->atochaModule->additionalSponsorship
- Event: atochaFinance.PuzzleDeposit
- Result: Storage->atochaFinance->atoFinanceLedger

### Solve a puzzle
- Action: Submission->atochaModule->answerPuzzle
- Event: atochaModule.AnswerCreated (ANSWER_HASH_IS_MATCH) && atochaModule.PuzzleStatusChange (PUZZLE_STATUS_IS_SOLVED) && atochaModule.AnnouncePuzzleChallengeDeadline
- Result: Storage->atochaModule->puzzleInfo

### Make an initial deposit of a challenge
- Action: Submission->atochaModule->commitChallenge
- Event:<br/>
atochaFinance.ChallengeDeposit && atochaFinance.ChallengeRaisePeriodDeadline<br/>
or<br/>
atochaFinance.ChallengeDeposit && atochaFinance.ChallengeRaisePeriodDeadline && council.Proposed<br/>
- Result: Storage->atochaFinance->puzzleChallengeInfo

### Make further deposit of a challenge
- Action: Submission->atochaModule->challengeCrowdloan
- Event:<br/>
atochaFinance.ChallengeDeposit<br/>
or<br/>
atochaFinance.ChallengeDeposit && atochaFinance.ChallengeStatusChange && council.Proposed<br/>
- Result: Storage->atochaFinance->puzzleChallengeInfo

### Claim for puzzle challenge deposit refund when challenge failed
- Action: Submission->atochaModule->challengePullOut
- Event: atochaFinance.ChallengeStatusChange
- Result: Storage->atochaFinance->puzzleChallengeInfo

### Claim for puzzle reward
- Action: Submission->atochaModule->takeAnswerReward
- Event: atochaFinance.TakeTokenReward && atochaFinance.TakePointReward
- Result: Storage->atochaFinance->puzzleChallengeInfo

### Claim for weekly point ranking reward
- Action: Submission->atochaFinance->applyPointReward
- Event: atochaFinance.applyPointReward
- Result: Storage->atochaFinance->pointExchangeInfo

### Check for a player's current points
- Result: Storage->atochaFinance->atoPointLedger
 
### Check for current point ranking for all players
- Result: Storage->atochaFinance->pointExchangeInfo

### Check for configuration parameters
- Storage->atochaModule->atoConfig()
- Storage->atochaFinance->atoConfig()

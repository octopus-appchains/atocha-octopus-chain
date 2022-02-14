### PreCreate a puzzle ###
Action: Submission->atochaFinace->preStorage<br/>
Event: atochaFinace.PreStorage<br/>
Result: Storage->atochaFinace->storageLedger<br/>

### Create a puzzle ###
Action: Submission->atochaModule->createPuzzle<br/>
Event: atochaModule.PuzzleCreated<br/>
Result: Storage->atochaModule->puzzleInfo<br/>

### Sponsor a puzzle ###
Action: Submission->atochaModule->additionalSponsorship<br/>
Event: atochaModule.AdditionalSponsorship<br/>
Result: Storage->atochaFinace->atoFinanceLedger<br/>

### Answer a puzzle ###
Action: Submission->atochaModule->answerPuzzle<br/>
Event: atochaModule.AnswerCreated (ANSWER_HASH_IS_MISMATCH or ANSWER_HASH_IS_MATCH)<br/>
Result: Storage->atochaModule->puzzleInfo<br/>

### Challenge a puzzle - create a challenge ###
Action: Submission->atochaModule->commitChallenge<br/>
Event: atochaFinace.ChallengeDeposit<br/>
Result: Storage->atochaFinace->puzzleChallengeInfo<br/>

### Challenge a puzzle - join a challenge ###
Action: Submission->atochaModule->challengeCrowdloan<br/>
Event: atochaFinace.ChallengeStatusChange<br/>
Result: Storage->atochaFinace->puzzleChallengeInfo<br/>

### Challenge a puzzle - refund challenge deposit ###
Action: Submission->atochaModule->challengePullOut<br/>
Event: atochaFinace.ChallengeStatusChange<br/>
Result: Storage->atochaFinace->puzzleChallengeInfo<br/>

### Claim for puzzle reward ###
Action: Submission->atochaModule->takeAnswerReward<br/>
Event: atochaFinace.TakeTokenReward && atochaFinace.TakePointReward<br/>
Result: Storage->atochaFinace->puzzleChallengeInfo<br/>

### Check point ranking ###
Result: Storage->atochaFinace->pointExchangeInfo<br/>

### Claim for weekly point ranking reward ###
Action: Submission->atochaFinace->applyPointReward<br/>
Event: atochaFinace.applyPointReward<br/>
Result: Storage->atochaFinace->pointExchangeInfo<br/>

